use std::sync::Arc;

use anchor_lang::Id;
use sablier_thread_program::{
    program::ThreadProgram,
    state::{Thread, VersionedThread},
};
use solana_sdk::signature::Keypair;

use crate::{error::PluginError, observers::Observers};

pub async fn load_all_accounts(observers: Arc<Observers>) -> Result<(), PluginError> {
    let client =
        anchor_client::Client::new(anchor_client::Cluster::Localnet, Arc::new(Keypair::new()));
    let accounts = client
        .program(ThreadProgram::id())?
        .accounts::<Thread>(vec![])
        .await?;

    for (key, thread) in accounts {
        if let Err(err) = observers
            .thread
            .clone()
            .observe_thread(VersionedThread::V1(thread), key, 0)
            .await
        {
            log::error!("Cannot observe account {key}: {err}");
        }
    }

    Ok(())
}
