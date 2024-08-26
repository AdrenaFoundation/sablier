use solana_sdk::pubkey::Pubkey;
use std::{collections::HashSet, ops::Deref};
use tokio::sync::RwLock;

use crate::observers::webhook::WebhookObserver;

use super::FromState;

#[derive(Default)]
pub struct WebhookState(RwLock<HashSet<Pubkey>>);

impl WebhookState {
    pub async fn add(&self, webhook: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state.insert(webhook);
    }
}

impl Deref for WebhookState {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<WebhookObserver> for WebhookState {
    fn from(state: &WebhookObserver) -> &Self {
        &state.webhooks
    }
}
