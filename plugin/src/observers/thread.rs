use std::{
    collections::HashSet,
    fmt::Debug,
    str::FromStr,
    sync::{atomic::AtomicU64, Arc},
};

use chrono::DateTime;
use log::info;
use pyth_solana_receiver_sdk::price_update::PriceFeedMessage;
use sablier_cron::Schedule;
use sablier_thread_program::state::{Equality, Trigger, TriggerContext, VersionedThread};
use solana_program::{clock::Clock, pubkey::Pubkey};

use crate::{error::PluginError, observers::state::PythThread, utils::get_oracle_key};

use super::state::{
    AccountThreads, Clocks, CronThreads, EpochThreads, NowThreads, PythThreads, SlotThreads,
    UpdatedAccounts,
};

#[derive(Default)]
pub struct ThreadObserver {
    // Map from slot numbers to the sysvar clock data for that slot.
    pub clocks: Clocks,

    // Integer tracking the current epoch.
    pub current_epoch: AtomicU64,

    // The set of threads with an account trigger.
    // Map from account pubkeys to the set of threads listening for an account update.
    pub account_threads: AccountThreads,

    // The set of threads with a cron trigger.
    // Map from unix timestamps to the list of threads scheduled for that moment.
    pub cron_threads: CronThreads,

    // The set of threads with a now trigger.
    pub now_threads: NowThreads,

    // The set of threads with a slot trigger.
    pub slot_threads: SlotThreads,

    // The set of threads with an epoch trigger.
    pub epoch_threads: EpochThreads,

    // The set of threads with a pyth trigger.
    pub pyth_threads: PythThreads,

    // The set of accounts that have updated.
    pub updated_accounts: UpdatedAccounts,
}

impl ThreadObserver {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn process_slot(self: Arc<Self>, slot: u64) -> HashSet<Pubkey> {
        let mut executable_threads = HashSet::new();

        // Drop old clocks.
        self.clocks.cleanup(slot).await;

        // Get the set of threads that were triggered by the current clock.
        let r_clocks = self.clocks.read().await;
        if let Some(clock) = r_clocks.get(&slot) {
            let mut w_cron_threads = self.cron_threads.write().await;
            w_cron_threads.retain(|target_timestamp, thread_pubkeys| {
                let is_due = clock.unix_timestamp >= *target_timestamp;
                if is_due {
                    self.current_epoch
                        .fetch_max(clock.epoch, std::sync::atomic::Ordering::Relaxed);
                    executable_threads.extend(thread_pubkeys.drain());
                }
                !is_due
            });
            drop(w_cron_threads);
        }
        drop(r_clocks);

        // Get the set of threads were triggered by an account update.
        let r_account_threads = self.account_threads.read().await;
        let mut w_updated_accounts = self.updated_accounts.write().await;
        w_updated_accounts.iter().for_each(|account_pubkey| {
            if let Some(thread_pubkeys) = r_account_threads.get(account_pubkey) {
                executable_threads.extend(thread_pubkeys);
            }
        });
        w_updated_accounts.clear();
        drop(r_account_threads);
        drop(w_updated_accounts);

        // Get the set of threads that were triggered by a slot update.
        let mut w_slot_threads = self.slot_threads.write().await;
        w_slot_threads.retain(|target_slot, thread_pubkeys| {
            let is_due = slot >= *target_slot;
            if is_due {
                executable_threads.extend(thread_pubkeys.drain());
            }
            !is_due
        });
        drop(w_slot_threads);

        // Get the set of threads that were trigger by an epoch update.
        let mut w_epoch_threads = self.epoch_threads.write().await;
        let current_epoch = self
            .current_epoch
            .load(std::sync::atomic::Ordering::Relaxed);
        w_epoch_threads.retain(|target_epoch, thread_pubkeys| {
            let is_due = current_epoch >= *target_epoch;
            if is_due {
                executable_threads.extend(thread_pubkeys.drain());
            }
            !is_due
        });
        drop(w_epoch_threads);

        // Get the set of immediate threads.
        let mut w_now_threads = self.now_threads.write().await;
        executable_threads.extend(w_now_threads.drain());

        executable_threads
    }

    pub async fn observe_clock(self: Arc<Self>, clock: Clock) {
        self.clocks.add(clock).await;
    }

    /// Move all threads listening to this account into the executable set.
    pub async fn observe_account(self: Arc<Self>, account_pubkey: Pubkey, _slot: u64) {
        if self.account_threads.contains(&account_pubkey).await {
            self.updated_accounts.add(account_pubkey).await;
        }
    }

    pub async fn observe_price_feed(
        self: Arc<Self>,
        account_pubkey: Pubkey,
        price_feed: PriceFeedMessage,
    ) {
        let r_pyth_threads = self.pyth_threads.read().await;
        if let Some(pyth_threads) = r_pyth_threads.get(&account_pubkey) {
            for pyth_thread in pyth_threads {
                match pyth_thread.equality {
                    Equality::GreaterThanOrEqual => {
                        if price_feed.price.ge(&pyth_thread.limit) {
                            self.now_threads.add(pyth_thread.thread_pubkey).await;
                        }
                    }
                    Equality::LessThanOrEqual => {
                        if price_feed.price.le(&pyth_thread.limit) {
                            self.now_threads.add(pyth_thread.thread_pubkey).await;
                        }
                    }
                }
            }
        }
        drop(r_pyth_threads);
    }

    pub async fn observe_thread(
        self: Arc<Self>,
        thread: VersionedThread,
        thread_pubkey: Pubkey,
        slot: u64,
    ) -> Result<(), PluginError> {
        // If the thread is paused, just return without indexing
        if thread.paused() {
            return Ok(());
        }

        info!("Indexing thread: {:?} slot: {}", thread_pubkey, slot);
        if thread.next_instruction().is_some() {
            // If the thread has a next instruction, index it as executable.
            self.now_threads.add(thread_pubkey).await;
        } else {
            // Otherwise, index the thread according to its trigger type.
            match thread.trigger() {
                Trigger::Account { address, .. } => {
                    // Index the thread by its trigger's account pubkey.
                    self.account_threads.add(address, thread_pubkey).await;

                    // Threads with account triggers might be immediately executable,
                    // Thus, we should attempt to execute these threads right away without for an account update.
                    self.now_threads.add(thread_pubkey).await;
                }
                Trigger::Cron { schedule, .. } => {
                    // Find a reference timestamp for calculating the thread's upcoming target time.
                    let reference_timestamp = match thread.exec_context() {
                        None => thread.created_at().unix_timestamp,
                        Some(exec_context) => match exec_context.trigger_context {
                            TriggerContext::Cron { started_at } => started_at,
                            _ => return Err(PluginError::InvalidExecContext),
                        },
                    };

                    // Index the thread to its target timestamp
                    match next_moment(reference_timestamp, schedule) {
                        None => {} // The thread does not have any upcoming scheduled target time
                        Some(target_timestamp) => {
                            self.cron_threads.add(target_timestamp, thread_pubkey).await
                        }
                    }
                }
                Trigger::Timestamp { unix_ts } => {
                    self.cron_threads.add(unix_ts, thread_pubkey).await
                }
                Trigger::Now => {
                    self.now_threads.add(thread_pubkey).await;
                }
                Trigger::Slot { slot } => {
                    self.slot_threads.add(slot, thread_pubkey).await;
                }
                Trigger::Epoch { epoch } => {
                    self.epoch_threads.add(epoch, thread_pubkey).await;
                }
                Trigger::Pyth {
                    feed_id,
                    equality,
                    limit,
                } => {
                    let pyth_thread = PythThread {
                        thread_pubkey,
                        equality,
                        limit,
                    };
                    let price_pubkey = get_oracle_key(0, feed_id);
                    self.pyth_threads.add(price_pubkey, pyth_thread).await;
                }
            }
        }

        Ok(())
    }
}

impl Debug for ThreadObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thread-observer")
    }
}

fn next_moment(after: i64, schedule: String) -> Option<i64> {
    match Schedule::from_str(&schedule) {
        Err(_) => None,
        Ok(schedule) => schedule
            .next_after(&DateTime::from_timestamp(after, 0).unwrap())
            .take()
            .map(|datetime| datetime.timestamp()),
    }
}
