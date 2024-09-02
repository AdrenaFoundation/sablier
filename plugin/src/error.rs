use solana_client::client_error::ClientError;
use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;
use solana_sdk::transaction::TransactionError;
use std::array::TryFromSliceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Cannot parse the account: {0}")]
    BinCodeError(#[from] bincode::Error),
    #[error("Cannot parse the account: {0}")]
    AnchorError(#[from] anchor_lang::error::Error),
    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] ClientError),
    #[error("The pubkey cannot be deserialized: {0}")]
    ParsingError(#[from] TryFromSliceError),
    #[error("RPC client has not reached min context slot")]
    MinContextSlotNotReached,
    #[error("Invalid exec context")]
    InvalidExecContext,
    #[error("Failed to send transaction")]
    FailedToSendTx,
    #[error("Tx failed simulation: {0} Logs: {1:#?}")]
    FailedToSimulateTx(TransactionError, Option<Vec<String>>),
}

impl From<PluginError> for GeyserPluginError {
    fn from(value: PluginError) -> Self {
        match value {
            PluginError::AnchorError(_) | PluginError::BinCodeError(_) => {
                GeyserPluginError::AccountsUpdateError {
                    msg: value.to_string(),
                }
            }
            _ => GeyserPluginError::Custom(Box::new(value)),
        }
    }
}
