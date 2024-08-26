use sablier_webhook_program::state::Webhook;
use solana_program::pubkey::Pubkey;
use std::{collections::HashSet, fmt::Debug, sync::Arc};

use super::state::WebhookState;

#[derive(Default)]
pub struct WebhookObserver {
    // The set of webhook that can be processed.
    pub webhooks: WebhookState,
}

impl WebhookObserver {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn observe_webhook(self: Arc<Self>, _webhook: Webhook, webhook_pubkey: Pubkey) {
        self.webhooks.add(webhook_pubkey).await;
    }

    pub async fn process_slot(self: Arc<Self>, _slot: u64) -> HashSet<Pubkey> {
        let mut executable_threads = HashSet::new();

        let mut w_webhooks = self.webhooks.write().await;

        executable_threads.extend(w_webhooks.drain());

        executable_threads
    }
}

impl Debug for WebhookObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "webhook-observer")
    }
}
