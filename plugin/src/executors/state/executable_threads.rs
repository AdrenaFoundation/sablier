use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
};

use log::info;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio::sync::RwLock;

use crate::{executors::tx::ExecutableThreadMetadata, pool_position::PoolPosition};

/// Number of times to retry a thread simulation.
static MAX_THREAD_SIMULATION_FAILURES: u32 = 5;

/// Number of slots to wait before trying to execute a thread while not in the pool.
static THREAD_TIMEOUT_WINDOW: u64 = 24;

/// The constant of the exponential backoff function.
static EXPONENTIAL_BACKOFF_CONSTANT: u32 = 2;

#[derive(Default)]
pub struct ExecutableThreads(RwLock<HashMap<Pubkey, ExecutableThreadMetadata>>, AtomicU64);

impl ExecutableThreads {
    pub async fn increment_simulation_failure(&self, thread_pubkey: Pubkey) {
        let mut w_state = self.0.write().await;
        w_state
            .entry(thread_pubkey)
            .and_modify(|metadata| metadata.simulation_failures += 1);
    }

    pub async fn get(&self, pool_position: &PoolPosition, slot: u64) -> Vec<(Pubkey, u64)> {
        // Get the set of thread pubkeys that are executable.
        // Note we parallelize using rayon because this work is CPU heavy.
        let r_state = self.0.read().await;

        if pool_position.current_position.is_none() && !pool_position.workers.is_empty() {
            // This worker is not in the pool. Get pubkeys of threads that are beyond the timeout window.
            r_state
                .iter()
                .filter(|(_pubkey, metadata)| slot > metadata.due_slot + THREAD_TIMEOUT_WINDOW)
                .filter(|(_pubkey, metadata)| slot >= exponential_backoff_threshold(metadata))
                .map(|(pubkey, metadata)| (*pubkey, metadata.due_slot))
                .collect()
        } else {
            // This worker is in the pool. Get pubkeys executable threads.
            r_state
                .iter()
                .filter(|(_pubkey, metadata)| slot >= exponential_backoff_threshold(metadata))
                .map(|(pubkey, metadata)| (*pubkey, metadata.due_slot))
                .collect()
        }
    }

    pub async fn remove_executed_threads(
        &self,
        executed_threads: &HashMap<Pubkey, (Signature, u64)>,
    ) {
        let mut w_state = self.0.write().await;

        for pubkey in executed_threads.keys() {
            w_state.remove(pubkey);
        }
    }

    pub async fn remove(&self, thread: &Pubkey) {
        let mut w_state = self.0.write().await;

        w_state.remove(thread);
    }

    pub async fn rebase_threads(&self, slot: u64, threads: &HashSet<Pubkey>) {
        // Index the provided threads as executable.
        let mut w_state = self.0.write().await;
        threads.iter().for_each(|pubkey| {
            w_state.insert(
                *pubkey,
                ExecutableThreadMetadata {
                    due_slot: slot,
                    simulation_failures: 0,
                },
            );
        });

        // Drop threads that cross the simulation failure threshold.
        w_state.retain(|thread_pubkey, metadata| {
            if metadata.simulation_failures > MAX_THREAD_SIMULATION_FAILURES {
                info!("Dropped thread: {thread_pubkey}");
                self.1.fetch_add(1, Ordering::Relaxed);
                false
            } else {
                true
            }
        });
        info!(
            "dropped_threads: {:?} executable_threads: {:?}",
            self.1.load(Ordering::Relaxed),
            *w_state
        );
    }

    pub async fn add(&self, retriable_threads: HashSet<(Pubkey, u64)>) {
        let mut w_state = self.0.write().await;
        for (pubkey, due_slot) in retriable_threads {
            w_state.insert(
                pubkey,
                ExecutableThreadMetadata {
                    due_slot,
                    simulation_failures: 0,
                },
            );
        }
    }
}

fn exponential_backoff_threshold(metadata: &ExecutableThreadMetadata) -> u64 {
    metadata.due_slot + EXPONENTIAL_BACKOFF_CONSTANT.pow(metadata.simulation_failures) as u64 - 1
}
