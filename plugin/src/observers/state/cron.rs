use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct CronThreads(RwLock<HashMap<i64, HashSet<Pubkey>>>);

impl CronThreads {
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

impl Deref for CronThreads {
    type Target = RwLock<HashMap<i64, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
