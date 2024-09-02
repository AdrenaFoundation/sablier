use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AccountThreads(RwLock<HashMap<Pubkey, HashSet<Pubkey>>>);

impl AccountThreads {
    pub async fn add(&self, address: Pubkey, thread_key: Pubkey) {
        let mut w_state = self.0.write().await;
        w_state
            .entry(address)
            .and_modify(|v| {
                v.insert(thread_key);
            })
            .or_insert(HashSet::from([thread_key]));
    }

    pub async fn contains(&self, account: &Pubkey) -> bool {
        let r_state = self.0.read().await;

        r_state.contains_key(account)
    }
}

impl Deref for AccountThreads {
    type Target = RwLock<HashMap<Pubkey, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct UpdatedAccounts(RwLock<HashSet<Pubkey>>);

impl UpdatedAccounts {
    pub async fn add(&self, account: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state.insert(account);
    }
}

impl Deref for UpdatedAccounts {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
