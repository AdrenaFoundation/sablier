use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError;
use std::array::TryFromSliceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Cannot parse the account: {0}")]
    BinCodeError(#[from] bincode::Error),
    #[error("Cannot parse the account: {0}")]
    AnchorError(#[from] anchor_lang::error::Error),
    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),
    #[error("The pubkey cannot be deserialized: {0}")]
    ParsingError(#[from] TryFromSliceError),
    #[error("Failed to parse Sablier account")]
    NotSablierAccount,
    #[error("RPC client has not reached min context slot")]
    MinContextSlotNotReached,
    #[error("Invalid exec context")]
    InvalidExecContext,
}

impl From<PluginError> for GeyserPluginError {
    fn from(value: PluginError) -> Self {
        match value {
            PluginError::AnchorError(_)
            | PluginError::BinCodeError(_)
            | PluginError::NotSablierAccount => GeyserPluginError::AccountsUpdateError {
                msg: value.to_string(),
            },
            PluginError::ParsingError(_)
            | PluginError::SolanaClientError(_)
            | PluginError::MinContextSlotNotReached
            | PluginError::InvalidExecContext => GeyserPluginError::Custom(Box::new(value)),
        }
    }
}
