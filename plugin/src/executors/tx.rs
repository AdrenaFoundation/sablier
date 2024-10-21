use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::Write,
    sync::{Arc, RwLock},
};

use bincode::serialize;
use log::info;
use sablier_network_program::state::{Pool, Registry, Snapshot, SnapshotFrame, Worker};
use sablier_thread_program::state::VersionedThread;
use solana_client::{
    nonblocking::{rpc_client::RpcClient, tpu_client::TpuClient},
    rpc_config::RpcSimulateTransactionConfig,
    tpu_client::TpuClientConfig,
};
use solana_geyser_plugin_interface::geyser_plugin_interface::Result as PluginResult;
use solana_quic_client::{QuicConfig, QuicConnectionManager, QuicPool};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use tokio::runtime::Runtime;

use crate::{
    config::PluginConfig, error::PluginError, pool_position::PoolPosition,
    utils::read_or_new_keypair,
};

use super::{
    state::{ExecutableThreads, RotationHistory, TransactionHistory},
    AccountGet,
};

/// TxExecutor
pub struct TxExecutor {
    pub config: PluginConfig,
    pub executable_threads: ExecutableThreads,
    // Temporary state for blacklisted threads (from old positions that were deleted and SLTP cleanups call not working)
    pub blacklisted_threads: RwLock<HashSet<Pubkey>>,
    pub transaction_history: TransactionHistory,
    pub rotation_history: RotationHistory,
    pub keypair: Keypair,
}

#[derive(Debug)]
pub struct ExecutableThreadMetadata {
    pub due_slot: u64,
    pub simulation_failures: u32,
}

#[derive(Debug)]
pub struct TransactionMetadata {
    pub due_slot: u64,
    pub sent_slot: u64,
    pub signature: Signature,
}

impl TxExecutor {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config: config.clone(),
            executable_threads: ExecutableThreads::default(),
            blacklisted_threads: RwLock::new(HashSet::new()),
            transaction_history: TransactionHistory::default(),
            rotation_history: RotationHistory::default(),
            keypair: read_or_new_keypair(config.keypath),
        }
    }

    pub async fn execute_txs(
        self: Arc<Self>,
        client: Arc<RpcClient>,
        thread_pubkeys: HashSet<Pubkey>,
        slot: u64,
        runtime: Arc<Runtime>,
    ) -> PluginResult<()> {
        info!(
            "blacklisted_threads: {:?}",
            self.blacklisted_threads.read().unwrap().len()
        );
        self.executable_threads
            .rebase_threads(slot, &thread_pubkeys)
            .await;

        // Process retries.
        self.clone().process_retries(client.clone(), slot).await;

        // Get self worker's position in the delegate pool.
        let worker_pubkey = Worker::pubkey(self.config.worker_id);
        if let Ok(pool_position) = (client.get::<Pool>(&Pool::pubkey(0)).await).map(|pool| {
            let workers = &mut pool.workers.clone();
            PoolPosition {
                current_position: pool
                    .workers
                    .iter()
                    .position(|k| k.eq(&worker_pubkey))
                    .map(|i| i as u64),
                workers: workers.make_contiguous().to_vec().clone(),
            }
        }) {
            info!("pool_position: {:?}", pool_position);

            // Rotate into the worker pool.
            if pool_position.current_position.is_none() {
                self.clone()
                    .execute_pool_rotate_txs(client.clone(), slot, pool_position.clone())
                    .await
                    .ok();
            }

            // Execute thread transactions.
            self.clone()
                .execute_thread_exec_txs(client.clone(), slot, pool_position, runtime.clone())
                .await
                .ok();
        }

        Ok(())
    }

    async fn process_retries(self: Arc<Self>, client: Arc<RpcClient>, slot: u64) {
        // Get transaction signatures and corresponding threads to check.
        let checkable_transactions = self.transaction_history.get_checkable_tx(slot).await;

        // Lookup transaction statuses and track which threads are successful / retriable.
        let mut failed_threads: HashSet<Pubkey> = HashSet::new();
        let mut retriable_threads: HashSet<(Pubkey, u64)> = HashSet::new();
        let mut successful_threads: HashSet<Pubkey> = HashSet::new();
        for data in checkable_transactions {
            match client
                .get_signature_status_with_commitment(
                    &data.signature,
                    CommitmentConfig::processed(),
                )
                .await
            {
                Err(_err) => {}
                Ok(status) => match status {
                    None => {
                        info!(
                            "Retrying thread: {:?} missing_signature: {:?}",
                            data.thread_pubkey, data.signature
                        );
                        retriable_threads.insert((data.thread_pubkey, data.due_slot));
                    }
                    Some(status) => match status {
                        Err(err) => {
                            info!(
                                "Thread failed: {:?} failed_signature: {:?} err: {:?}",
                                data.thread_pubkey, data.signature, err
                            );
                            failed_threads.insert(data.thread_pubkey);
                        }
                        Ok(()) => {
                            successful_threads.insert(data.thread_pubkey);
                        }
                    },
                },
            }
        }

        // Requeue retriable threads and drop transactions from history.
        self.transaction_history
            .clean(&failed_threads, &retriable_threads, &successful_threads)
            .await;
        self.executable_threads.add(retriable_threads).await;
    }

    async fn execute_pool_rotate_txs(
        self: Arc<Self>,
        client: Arc<RpcClient>,
        slot: u64,
        pool_position: PoolPosition,
    ) -> Result<(), PluginError> {
        let should_attempt = self
            .rotation_history
            .should_attempt(client.clone(), slot)
            .await;

        if !should_attempt {
            return Ok(());
        }
        let registry = client.get::<Registry>(&Registry::pubkey()).await?;
        let snapshot_pubkey = Snapshot::pubkey(registry.current_epoch);
        let snapshot_frame_pubkey = SnapshotFrame::pubkey(snapshot_pubkey, self.config.worker_id);
        if let Ok(snapshot) = client.get::<Snapshot>(&snapshot_pubkey).await {
            if let Ok(snapshot_frame) = client.get::<SnapshotFrame>(&snapshot_frame_pubkey).await {
                if let Some(tx) = crate::builders::build_pool_rotation_tx(
                    client.clone(),
                    &self.keypair,
                    pool_position,
                    registry,
                    snapshot,
                    snapshot_frame,
                    self.config.worker_id,
                )
                .await
                {
                    self.clone().simulate_tx(&tx).await?;
                    self.clone().submit_tx(&tx).await?;
                    self.rotation_history
                        .add(TransactionMetadata {
                            due_slot: slot,
                            sent_slot: slot,
                            signature: tx.signatures[0],
                        })
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn execute_thread_exec_txs(
        self: Arc<Self>,
        client: Arc<RpcClient>,
        observed_slot: u64,
        pool_position: PoolPosition,
        runtime: Arc<Runtime>,
    ) -> PluginResult<()> {
        let executable_threads = self
            .executable_threads
            .get(&pool_position, observed_slot)
            .await;
        if executable_threads.is_empty() {
            return Ok(());
        }

        // Build transactions in parallel.
        // Note we parallelize using tokio because this work is IO heavy (RPC simulation calls).
        let tasks: Vec<_> = executable_threads
            .iter()
            .map(|(thread_pubkey, due_slot)| {
                runtime.spawn(self.clone().try_build_thread_exec_tx(
                    client.clone(),
                    observed_slot,
                    *due_slot,
                    *thread_pubkey,
                ))
            })
            .collect();
        let mut executed_threads: HashMap<Pubkey, (Signature, u64)> = HashMap::new();

        // Serialize to wire transactions.
        let wire_txs = futures::future::join_all(tasks)
            .await
            .iter()
            .filter_map(|res| match res {
                Err(_err) => None,
                Ok(res) => match res {
                    None => None,
                    Some((pubkey, tx, due_slot)) => {
                        executed_threads.insert(*pubkey, (tx.signatures[0], *due_slot));
                        Some(tx)
                    }
                },
            })
            .map(|tx| serialize(tx).unwrap())
            .collect::<Vec<Vec<u8>>>();

        // Batch submit transactions to the leader.
        match get_tpu_client()
            .await
            .try_send_wire_transaction_batch(wire_txs)
            .await
        {
            Err(err) => {
                info!("Failed to sent transaction batch: {:?}", err);
            }
            Ok(()) => {
                self.executable_threads
                    .remove_executed_threads(&executed_threads)
                    .await;
                self.transaction_history
                    .add(observed_slot, executed_threads)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn try_build_thread_exec_tx(
        self: Arc<Self>,
        client: Arc<RpcClient>,
        observed_slot: u64,
        due_slot: u64,
        thread_pubkey: Pubkey,
    ) -> Option<(Pubkey, Transaction, u64)> {
        let thread = match client.clone().get::<VersionedThread>(&thread_pubkey).await {
            Err(_err) => {
                self.executable_threads
                    .increment_simulation_failure(thread_pubkey)
                    .await;
                return None;
            }
            Ok(thread) => thread,
        };

        // Exit early if the thread has been executed after it became due.
        if let Some(exec_context) = thread.exec_context() {
            if exec_context.last_exec_at.gt(&due_slot) {
                // Drop thread from the executable set to avoid bloat.
                self.executable_threads.remove(&thread_pubkey).await;
                return None;
            }
        }

        // check if the thread is blacklisted
        if self
            .blacklisted_threads
            .read()
            .unwrap()
            .contains(&thread_pubkey)
        {
            return None;
        } else if let Ok((tx, blacklisted)) = crate::builders::build_thread_exec_tx(
            client.clone(),
            &self.keypair,
            due_slot,
            thread,
            thread_pubkey,
            self.config.worker_id,
        )
        .await
        {
            if let Some(blacklisted) = blacklisted {
                self.blacklisted_threads
                    .write()
                    .unwrap()
                    .insert(blacklisted);
                None
            } else if let Some(tx) = tx {
                if !self
                    .transaction_history
                    .is_duplicate_tx(observed_slot, thread_pubkey, &tx)
                    .await
                {
                    Some((thread_pubkey, tx, due_slot))
                } else {
                    None
                }
            } else {
                self.executable_threads
                    .increment_simulation_failure(thread_pubkey)
                    .await;
                None
            }
        } else {
            None
        }
    }

    async fn simulate_tx(self: Arc<Self>, tx: &Transaction) -> Result<Transaction, PluginError> {
        let response =
            RpcClient::new_with_commitment(LOCAL_RPC_URL.into(), CommitmentConfig::processed())
                .simulate_transaction_with_config(
                    tx,
                    RpcSimulateTransactionConfig {
                        replace_recent_blockhash: false,
                        commitment: Some(CommitmentConfig::processed()),
                        ..RpcSimulateTransactionConfig::default()
                    },
                )
                .await?;

        match response.value.err {
            Some(err) => Err(PluginError::FailedToSimulateTx(err, response.value.logs)),
            None => Ok(tx.clone()),
        }
    }

    async fn submit_tx(self: Arc<Self>, tx: &Transaction) -> Result<Transaction, PluginError> {
        if !get_tpu_client().await.send_transaction(tx).await {
            return Err(PluginError::FailedToSendTx);
        }
        Ok(tx.clone())
    }
}

impl Debug for TxExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx-executor")
    }
}

static LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";
static LOCAL_WEBSOCKET_URL: &str = "ws://127.0.0.1:8900";

// Do not use a static ref here.
// -> The quic connections are dropped only when TpuClient is dropped
async fn get_tpu_client() -> TpuClient<QuicPool, QuicConnectionManager, QuicConfig> {
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        LOCAL_RPC_URL.into(),
        CommitmentConfig::processed(),
    ));

    TpuClient::new(
        "sablier",
        rpc_client,
        LOCAL_WEBSOCKET_URL,
        TpuClientConfig { fanout_slots: 24 },
    )
    .await
    .unwrap()
}
