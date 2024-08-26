use solana_sdk::clock::Clock;
use std::{collections::HashMap, ops::Deref};
use tokio::sync::RwLock;

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct ClockState(RwLock<HashMap<u64, Clock>>);

impl ClockState {
    /// Drop old clocks.
    pub async fn cleanup(&self, current_slot: u64) {
        let mut w_state = self.0.write().await;
        w_state.retain(|cached_slot, _clock| *cached_slot >= current_slot);
    }

    pub async fn add(&self, clock: Clock) {
        let mut w_state = self.0.write().await;
        w_state.insert(clock.slot, clock.clone());
    }
}

impl Deref for ClockState {
    type Target = RwLock<HashMap<u64, Clock>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for ClockState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.clocks
    }
}
