use std::{collections::HashMap, ops::Deref};

use solana_sdk::clock::Clock;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Clocks(RwLock<HashMap<u64, Clock>>);

impl Clocks {
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

impl Deref for Clocks {
    type Target = RwLock<HashMap<u64, Clock>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
