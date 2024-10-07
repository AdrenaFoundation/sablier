use std::{fmt::Debug, sync::Arc};

use anchor_lang::AccountDeserialize;
use log::info;
use sablier_thread_program::state::VersionedThread;
use solana_client::rpc_client::RpcClient;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, ReplicaAccountInfoVersions, Result as PluginResult, SlotStatus,
};
use solana_sdk::pubkey::Pubkey;
use tokio::runtime::{Builder, Runtime};

use crate::{
    config::PluginConfig,
    error::PluginError,
    events::{AccountUpdate, AccountUpdateEvent},
    executors::Executors,
    observers::Observers,
};

pub struct SablierPlugin {
    pub inner: Arc<Inner>,
}

impl Debug for SablierPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inner: {:?}", self.inner)
    }
}

static LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";

#[derive(Debug)]
pub struct Inner {
    pub config: PluginConfig,
    pub executors: Arc<Executors>,
    pub observers: Arc<Observers>,
    pub runtime: Arc<Runtime>,
}

impl GeyserPlugin for SablierPlugin {
    fn name(&self) -> &'static str {
        "sablier-plugin"
    }

    fn on_load(&mut self, config_file: &str, _is_reload: bool) -> PluginResult<()> {
        solana_logger::setup_with_default("info");
        info!(
            "sablier-plugin crate-info - spec: {}, geyser_interface_version: {}, rustc: {}",
            env!("SPEC"),
            env!("GEYSER_INTERFACE_VERSION"),
            env!("RUSTC_VERSION")
        );
        info!("Loading snapshot...");
        let config = PluginConfig::read_from(config_file)?;
        *self = SablierPlugin::new_from_config(config);

        info!("Fetch existing Thread pdas...");
        let existing_thread_pdas = self.fetch_existing_threads()?;

        info!("Add fetched Thread pdas to observers...");
        let observers = self.inner.observers.clone();
        self.inner.clone().spawn(move |inner| async move {
            for (pubkey, thread) in existing_thread_pdas {
                observers
                    .thread
                    .clone()
                    .observe_thread(thread, pubkey, 0)
                    .await
                    .ok();
            }
            Ok(())
        });

        Ok(())
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> PluginResult<()> {
        // Parse account info.
        let account_update = AccountUpdate::from(account);

        // Process event on tokio task.
        self.inner.clone().spawn(|inner| async move {
            // Send all account updates to the thread observer for account listeners.
            // Only process account updates if we're past the startup phase.
            if !is_startup {
                inner
                    .observers
                    .thread
                    .clone()
                    .observe_account(account_update.key, slot)
                    .await;
            }

            if let Some(event) = account_update.event {
                // Process specific update events.
                match event {
                    AccountUpdateEvent::Clock { clock } => {
                        inner.observers.thread.clone().observe_clock(clock).await;
                    }
                    AccountUpdateEvent::Thread { thread } => {
                        inner
                            .observers
                            .thread
                            .clone()
                            .observe_thread(*thread, account_update.key, slot)
                            .await
                            .ok();
                    }
                    AccountUpdateEvent::PriceFeed { price_feed } => {
                        inner
                            .observers
                            .thread
                            .clone()
                            .observe_price_feed(account_update.key, price_feed)
                            .await;
                    }
                }
            }

            Ok(())
        });
        Ok(())
    }

    fn notify_end_of_startup(&self) -> PluginResult<()> {
        info!("Snapshot loaded");
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        _parent: Option<u64>,
        status: SlotStatus,
    ) -> PluginResult<()> {
        self.inner.clone().spawn(|inner| async move {
            if let SlotStatus::Processed = status {
                inner
                    .executors
                    .clone()
                    .process_slot(inner.observers.clone(), slot, inner.runtime.clone())
                    .await?;
            }
            Ok(())
        });
        Ok(())
    }
}

impl SablierPlugin {
    fn new_from_config(config: PluginConfig) -> Self {
        let runtime = build_runtime(config.clone());
        let observers = Arc::new(Observers::new());
        let executors = Arc::new(Executors::new(config.clone()));
        Self {
            inner: Arc::new(Inner {
                config,
                executors,
                observers,
                runtime,
            }),
        }
    }

    /// Fetch existing threads from the chain, and return them as a list of (pubkey, thread) pairs.
    /// Goal of this is to catch up on any existing threads that were created before the plugin was loaded.
    fn fetch_existing_threads(&self) -> PluginResult<Vec<(Pubkey, VersionedThread)>> {
        // Using localhost will use the RPC client from the validator.
        let rpc_client = RpcClient::new(LOCAL_RPC_URL.to_string());
        let program_id = sablier_thread_program::ID;

        let accounts = rpc_client
            .get_program_accounts(&program_id)
            .map_err(|e| PluginError::from(e))?;

        Ok(accounts
            .into_iter()
            .filter_map(|(pubkey, account)| {
                VersionedThread::try_deserialize(&mut account.data.as_slice())
                    .ok()
                    .map(|thread| (pubkey, thread))
            })
            .collect())
    }
}

impl Default for SablierPlugin {
    fn default() -> Self {
        Self::new_from_config(PluginConfig::default())
    }
}

impl Inner {
    fn spawn<F: std::future::Future<Output = PluginResult<()>> + Send + 'static>(
        self: Arc<Self>,
        f: impl FnOnce(Arc<Self>) -> F,
    ) {
        self.runtime.spawn(f(self.clone()));
    }
}

fn build_runtime(config: PluginConfig) -> Arc<Runtime> {
    Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .thread_name("sablier-plugin")
            .worker_threads(config.thread_count)
            .max_blocking_threads(config.thread_count)
            .build()
            .unwrap(),
    )
}
