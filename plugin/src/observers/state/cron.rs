use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use tokio::sync::RwLock;

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct CronState(RwLock<HashMap<i64, HashSet<Pubkey>>>);

impl CronState {
    pub async fn add(&self, timestamp: i64, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state
            .entry(timestamp)
            .and_modify(|v| {
                v.insert(thread_key);
            })
            .or_insert(HashSet::from([thread_key]));
    }
}

impl Deref for CronState {
    type Target = RwLock<HashMap<i64, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for CronState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.cron_threads
    }
}
