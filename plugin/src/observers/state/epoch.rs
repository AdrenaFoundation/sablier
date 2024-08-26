use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct EpochState(RwLock<HashMap<u64, HashSet<Pubkey>>>);

impl EpochState {
    pub async fn add(&self, epoch: u64, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state
            .entry(epoch)
            .and_modify(|v| {
                v.insert(thread_key);
            })
            .or_insert(HashSet::from([thread_key]));
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, HashMap<u64, HashSet<Pubkey>>> {
        self.0.write().await
    }
}

impl Deref for EpochState {
    type Target = RwLock<HashMap<u64, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for EpochState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.epoch_threads
    }
}
