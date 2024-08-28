use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use tokio::sync::RwLock;

use crate::executors::tx::TransactionMetadata;

/// The number of slots to wait since the last rotation attempt.
static ROTATION_CONFIRMATION_PERIOD: u64 = 16;

#[derive(Default)]
pub struct RotationHistory(RwLock<Option<TransactionMetadata>>);

impl RotationHistory {
    pub async fn add(&self, tx: TransactionMetadata) {
        let mut w_state = self.0.write().await;
        *w_state = Some(tx);
    }

    pub async fn should_attempt(&self, client: Arc<RpcClient>, slot: u64) -> bool {
        let r_state = self.0.read().await;
        log::info!("Rotation history {:?}", r_state);

        let Some(rotation_history) = r_state.as_ref() else {
            return true;
        };

        if slot
            <= rotation_history
                .sent_slot
                .saturating_add(ROTATION_CONFIRMATION_PERIOD)
        {
            return false;
        }

        let Ok(Some(status)) = client
            .get_signature_status(&rotation_history.signature)
            .await
        else {
            return true;
        };

        status.is_err()
    }
}

/*
        let r_rotation_history = self.rotation_history.read().await;
        log::info!("Rotation history {:?}", r_rotation_history);
        let should_attempt = if r_rotation_history.is_some() {
            let rotation_history = r_rotation_history.as_ref().unwrap();
            if slot
                > rotation_history
                    .sent_slot
                    .checked_add(ROTATION_CONFIRMATION_PERIOD)
                    .unwrap()
            {
                if let Ok(Some(status)) = client
                    .get_signature_status(&rotation_history.signature)
                    .await
                {
                    status.is_err()
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            true
        };

*/
