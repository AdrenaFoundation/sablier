use std::{collections::HashSet, ops::Deref};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct NowThreads(RwLock<HashSet<Pubkey>>);

impl NowThreads {
    pub async fn add(&self, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;
        w_state.insert(thread_key);
    }
}

impl Deref for NowThreads {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
