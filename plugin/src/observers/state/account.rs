use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use tokio::sync::RwLock;

use crate::observers::thread::ThreadObserver;

use super::FromState;

#[derive(Default)]
pub struct AccountState(RwLock<HashMap<Pubkey, HashSet<Pubkey>>>);

impl AccountState {
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

impl Deref for AccountState {
    type Target = RwLock<HashMap<Pubkey, HashSet<Pubkey>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for AccountState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.account_threads
    }
}

#[derive(Default)]
pub struct UpdatedAccountState(RwLock<HashSet<Pubkey>>);

impl UpdatedAccountState {
    pub async fn add(&self, account: Pubkey) {
        let mut w_state = self.0.write().await;

        w_state.insert(account);
    }
}

impl Deref for UpdatedAccountState {
    type Target = RwLock<HashSet<Pubkey>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromState<ThreadObserver> for UpdatedAccountState {
    fn from(state: &ThreadObserver) -> &Self {
        &state.updated_accounts
    }
}
