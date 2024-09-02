use std::collections::{HashMap, HashSet};

use log::info;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use tokio::sync::RwLock;

use crate::executors::tx::TransactionMetadata;

/// Number of slots to wait before checking for a confirmed transaction.
static TRANSACTION_CONFIRMATION_PERIOD: u64 = 24;

pub struct CheckableTransaction {
    pub due_slot: u64,
    pub thread_pubkey: Pubkey,
    pub signature: Signature,
}

#[derive(Default)]
pub struct TransactionHistory(RwLock<HashMap<Pubkey, TransactionMetadata>>);

impl TransactionHistory {
    pub async fn is_duplicate_tx(
        &self,
        slot: u64,
        thread_pubkey: Pubkey,
        tx: &Transaction,
    ) -> bool {
        let r_state = self.0.read().await;
        if let Some(metadata) = r_state.get(&thread_pubkey) {
            if metadata.signature.eq(&tx.signatures[0]) && metadata.sent_slot.le(&slot) {
                return true;
            }
        }
        false
    }

    pub async fn get_checkable_tx(&self, slot: u64) -> Vec<CheckableTransaction> {
        let r_state = self.0.read().await;
        r_state
            .iter()
            .filter(|(_, metadata)| slot > metadata.sent_slot + TRANSACTION_CONFIRMATION_PERIOD)
            .map(|(pubkey, metadata)| CheckableTransaction {
                due_slot: metadata.due_slot,
                thread_pubkey: *pubkey,
                signature: metadata.signature,
            })
            .collect()
    }

    pub async fn add(
        &self,
        observed_slot: u64,
        executed_threads: HashMap<Pubkey, (Signature, u64)>,
    ) {
        let mut w_state = self.0.write().await;

        for (pubkey, (signature, due_slot)) in executed_threads {
            w_state.insert(
                pubkey,
                TransactionMetadata {
                    due_slot,
                    sent_slot: observed_slot,
                    signature,
                },
            );
        }
    }

    pub async fn clean(
        &self,
        failed_threads: &HashSet<Pubkey>,
        retriable_threads: &HashSet<(Pubkey, u64)>,
        successful_threads: &HashSet<Pubkey>,
    ) {
        let mut w_state = self.0.write().await;
        for pubkey in successful_threads {
            w_state.remove(pubkey);
        }
        for pubkey in failed_threads {
            w_state.remove(pubkey);
        }
        for (pubkey, _) in retriable_threads {
            w_state.remove(pubkey);
        }
        info!("transaction_history: {:?}", *w_state);
    }
}
