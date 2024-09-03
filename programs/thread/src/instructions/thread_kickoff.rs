use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    str::FromStr,
};

use anchor_lang::prelude::*;
use chrono::DateTime;
use sablier_cron::Schedule;
use sablier_network_program::state::{Worker, WorkerAccount};
use sablier_utils::{
    pyth::{self, PriceUpdateV2},
    thread::Trigger,
};

use crate::{constants::*, errors::*, state::*};

/// Accounts required by the `thread_kickoff` instruction.
#[derive(Accounts)]
pub struct ThreadKickoff<'info> {
    /// The signatory.
    #[account(mut)]
    pub signatory: Signer<'info>,

    /// The thread to kickoff.
    #[account(
        mut,
        seeds = [
            SEED_THREAD,
            thread.authority.as_ref(),
            thread.id.as_slice(),
            thread.domain.as_ref().unwrap_or(&Vec::new()).as_slice()
        ],
        bump = thread.bump,
        constraint = !thread.paused @ SablierError::ThreadPaused,
        constraint = thread.next_instruction.is_none() @ SablierError::ThreadBusy,
    )]
    pub thread: Account<'info, Thread>,

    /// The worker.
    #[account(address = worker.pubkey())]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<ThreadKickoff>) -> Result<()> {
    // Get accounts.
    let signatory = &mut ctx.accounts.signatory;
    let thread = &mut ctx.accounts.thread;
    let clock = Clock::get()?;

    match &thread.trigger {
        Trigger::Account {
            address,
            offset,
            size,
        } => {
            // Verify proof that account data has been updated.
            match ctx.remaining_accounts.first() {
                None => {
                    return Err(SablierError::TriggerConditionFailed.into());
                }
                Some(account_info) => {
                    // Verify the remaining account is the account this thread is listening for.
                    require!(
                        address.eq(account_info.key),
                        SablierError::TriggerConditionFailed
                    );

                    // Begin computing the data hash of this account.
                    let mut hasher = DefaultHasher::new();
                    let data = &account_info.try_borrow_data()?;
                    let offset = *offset as usize;
                    let range_end = offset + *size as usize;
                    if data.len().gt(&range_end) {
                        data[offset..range_end].hash(&mut hasher);
                    } else {
                        data[offset..].hash(&mut hasher)
                    }
                    let data_hash = hasher.finish();

                    // Verify the data hash is different than the prior data hash.
                    if let Some(exec_context) = thread.exec_context {
                        match exec_context.trigger_context {
                            TriggerContext::Account {
                                data_hash: prior_data_hash,
                            } => {
                                require!(
                                    data_hash.ne(&prior_data_hash),
                                    SablierError::TriggerConditionFailed
                                )
                            }
                            _ => return Err(SablierError::InvalidThreadState.into()),
                        }
                    }

                    // Set a new exec context with the new data hash and slot number.
                    thread.exec_context = Some(ExecContext {
                        exec_index: 0,
                        execs_since_reimbursement: 0,
                        execs_since_slot: 0,
                        last_exec_at: clock.slot,
                        trigger_context: TriggerContext::Account { data_hash },
                    })
                }
            }
        }
        Trigger::Cron {
            schedule,
            skippable,
        } => {
            // Get the reference timestamp for calculating the thread's scheduled target timestamp.
            let reference_timestamp = match thread.exec_context {
                None => thread.created_at.unix_timestamp,
                Some(exec_context) => match exec_context.trigger_context {
                    TriggerContext::Cron { started_at } => started_at,
                    _ => return Err(SablierError::InvalidThreadState.into()),
                },
            };

            // Verify the current timestamp is greater than or equal to the threshold timestamp.
            let threshold_timestamp = next_timestamp(reference_timestamp, schedule)
                .ok_or(SablierError::TriggerConditionFailed)?;
            msg!(
                "Threshold timestamp: {}, clock timestamp: {}",
                threshold_timestamp,
                clock.unix_timestamp
            );
            require!(
                clock.unix_timestamp.ge(&threshold_timestamp),
                SablierError::TriggerConditionFailed
            );

            // If the schedule is marked as skippable, set the started_at of the exec context to be the current timestamp.
            // Otherwise, the exec context must iterate through each scheduled kickoff moment.
            let started_at = if *skippable {
                clock.unix_timestamp
            } else {
                threshold_timestamp
            };

            // Set the exec context.
            thread.exec_context = Some(ExecContext {
                exec_index: 0,
                execs_since_reimbursement: 0,
                execs_since_slot: 0,
                last_exec_at: clock.slot,
                trigger_context: TriggerContext::Cron { started_at },
            });
        }
        Trigger::Now => {
            // Set the exec context.
            require!(
                thread.exec_context.is_none(),
                SablierError::InvalidThreadState
            );
            thread.exec_context = Some(ExecContext {
                exec_index: 0,
                execs_since_reimbursement: 0,
                execs_since_slot: 0,
                last_exec_at: clock.slot,
                trigger_context: TriggerContext::Now,
            });
        }
        Trigger::Slot { slot } => {
            require!(clock.slot.ge(slot), SablierError::TriggerConditionFailed);
            thread.exec_context = Some(ExecContext {
                exec_index: 0,
                execs_since_reimbursement: 0,
                execs_since_slot: 0,
                last_exec_at: clock.slot,
                trigger_context: TriggerContext::Slot { started_at: *slot },
            });
        }
        Trigger::Epoch { epoch } => {
            require!(clock.epoch.ge(epoch), SablierError::TriggerConditionFailed);
            thread.exec_context = Some(ExecContext {
                exec_index: 0,
                execs_since_reimbursement: 0,
                execs_since_slot: 0,
                last_exec_at: clock.slot,
                trigger_context: TriggerContext::Epoch { started_at: *epoch },
            })
        }
        Trigger::Timestamp { unix_ts } => {
            require!(
                clock.unix_timestamp.ge(unix_ts),
                SablierError::TriggerConditionFailed
            );
            thread.exec_context = Some(ExecContext {
                exec_index: 0,
                execs_since_reimbursement: 0,
                execs_since_slot: 0,
                last_exec_at: clock.slot,
                trigger_context: TriggerContext::Timestamp {
                    started_at: *unix_ts,
                },
            })
        }
        Trigger::Pyth {
            feed_id,
            equality,
            limit,
        } => {
            // Verify price limit has been reached.
            match ctx.remaining_accounts.first() {
                None => {
                    return Err(SablierError::TriggerConditionFailed.into());
                }
                Some(account_info) => {
                    require_keys_eq!(
                        *account_info.owner,
                        pyth::ID,
                        SablierError::TriggerConditionFailed
                    );
                    const STALENESS_THRESHOLD: u64 = 60; // staleness threshold in seconds
                    let price_update =
                        PriceUpdateV2::try_deserialize(&mut account_info.data.borrow().as_ref())?;

                    let current_price = price_update.get_price_no_older_than(
                        &Clock::get()?,
                        STALENESS_THRESHOLD,
                        feed_id,
                    )?;

                    match equality {
                        Equality::GreaterThanOrEqual => {
                            require!(
                                current_price.price.ge(limit),
                                SablierError::TriggerConditionFailed
                            );
                            thread.exec_context = Some(ExecContext {
                                exec_index: 0,
                                execs_since_reimbursement: 0,
                                execs_since_slot: 0,
                                last_exec_at: clock.slot,
                                trigger_context: TriggerContext::Pyth {
                                    price: current_price.price,
                                },
                            });
                        }
                        Equality::LessThanOrEqual => {
                            require!(
                                current_price.price.le(limit),
                                SablierError::TriggerConditionFailed
                            );
                            thread.exec_context = Some(ExecContext {
                                exec_index: 0,
                                execs_since_reimbursement: 0,
                                execs_since_slot: 0,
                                last_exec_at: clock.slot,
                                trigger_context: TriggerContext::Pyth {
                                    price: current_price.price,
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    // If we make it here, the trigger is active. Update the next instruction and be done.
    if let Some(kickoff_instruction) = thread.instructions.first() {
        thread.next_instruction = Some(kickoff_instruction.clone());
    }

    // Realloc the thread account
    thread.realloc_account()?;

    // Reimburse signatory for transaction fee.
    thread.sub_lamports(TRANSACTION_BASE_FEE_REIMBURSEMENT)?;
    signatory.add_lamports(TRANSACTION_BASE_FEE_REIMBURSEMENT)?;

    Ok(())
}

fn next_timestamp(after: i64, schedule: &str) -> Option<i64> {
    Schedule::from_str(schedule)
        .unwrap()
        .next_after(&DateTime::from_timestamp(after, 0).unwrap())
        .take()
        .map(|datetime| datetime.timestamp())
}
