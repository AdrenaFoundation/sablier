use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct SlotState(RwLock<HashMap<u64, HashSet<Pubkey>>>);

impl SlotState {
    pub async fn add(&self, slot: u64, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state
            .entry(slot)
            .and_modify(|v| {
                v.insert(thread_key);
            })
            .or_insert(HashSet::from([thread_key]));
    }
}

impl Deref for SlotState {
    type Target = RwLock<HashMap<u64, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for SlotState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.slot_threads
    }
}
