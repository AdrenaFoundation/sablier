use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::{RwLock, RwLockWriteGuard};

#[derive(Default)]
pub struct EpochThreads(RwLock<HashMap<u64, HashSet<Pubkey>>>);

impl EpochThreads {
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

impl Deref for EpochThreads {
    type Target = RwLock<HashMap<u64, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
