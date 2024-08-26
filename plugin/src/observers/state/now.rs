use solana_sdk::pubkey::Pubkey;
use std::{collections::HashSet, ops::Deref};
use tokio::sync::RwLock;

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct NowState(RwLock<HashSet<Pubkey>>);

impl NowState {
    pub async fn add(&self, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;
        w_state.insert(thread_key);
    }
}

impl Deref for NowState {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for NowState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.now_threads
    }
}
