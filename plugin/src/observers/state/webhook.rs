use std::{collections::HashSet, ops::Deref};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Webhooks(RwLock<HashSet<Pubkey>>);

impl Webhooks {
    pub async fn add(&self, webhook: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state.insert(webhook);
    }
}

impl Deref for Webhooks {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
