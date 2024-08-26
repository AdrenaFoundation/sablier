use sablier_utils::thread::Equality;
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use tokio::sync::RwLock;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PythThread {
    pub thread_pubkey: Pubkey,
    pub equality: Equality,
    pub limit: i64,
}

#[derive(Default)]
pub struct PythState(RwLock<HashMap<Pubkey, HashSet<PythThread>>>);

impl PythState {
    pub async fn add(&self, price_key: Pubkey, data: PythThread) {
        let mut w_state = self.0.write().await;

        w_state
            .entry(price_key)
            .and_modify(|v| {
                v.insert(data.to_owned());
            })
            .or_insert(HashSet::from([data]));
    }
}

impl Deref for PythState {
    type Target = RwLock<HashMap<Pubkey, HashSet<PythThread>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
