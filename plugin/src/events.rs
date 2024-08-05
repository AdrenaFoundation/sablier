use anchor_lang::{AccountDeserialize, Discriminator};
use bincode::deserialize;
use pyth_solana_receiver_sdk::price_update::{PriceFeedMessage, PriceUpdateV2};
use sablier_thread_program::state::{Thread, VersionedThread};
use sablier_webhook_program::state::Webhook;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPluginError, ReplicaAccountInfo,
};
use solana_program::{clock::Clock, pubkey::Pubkey, sysvar};

#[derive(Debug)]
pub enum AccountUpdateEvent {
    Clock { clock: Clock },
    Thread { thread: VersionedThread },
    PriceFeed { price_feed: PriceFeedMessage },
    Webhook { webhook: Webhook },
}

impl TryFrom<&mut ReplicaAccountInfo<'_>> for AccountUpdateEvent {
    type Error = GeyserPluginError;
    fn try_from(account_info: &mut ReplicaAccountInfo) -> Result<Self, Self::Error> {
        // Parse pubkeys.
        let account_pubkey = Pubkey::try_from(account_info.pubkey).unwrap();
        let owner_pubkey = Pubkey::try_from(account_info.owner).unwrap();

        // If the account is the sysvar clock, parse it.
        if account_pubkey.eq(&sysvar::clock::ID) {
            return Ok(AccountUpdateEvent::Clock {
                clock: deserialize::<Clock>(account_info.data).map_err(|_e| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: "Failed to parsed sysvar clock account".into(),
                    }
                })?,
            });
        }

        // If the account belongs to the thread v1 program, parse it.
        if owner_pubkey.eq(&sablier_thread_program::ID) && account_info.data.len() > 8 {
            let d = &account_info.data[..8];
            if d.eq(&Thread::discriminator()) {
                return Ok(AccountUpdateEvent::Thread {
                    thread: VersionedThread::V1(
                        Thread::try_deserialize(&mut account_info.data).map_err(|_| {
                            GeyserPluginError::AccountsUpdateError {
                                msg: "Failed to parse Sablier thread v1 account".into(),
                            }
                        })?,
                    ),
                });
            }
        }

        // If the account belongs to Pyth, attempt to parse it.
        if owner_pubkey.eq(&pyth_solana_receiver_sdk::ID) {
            let price_account =
                PriceUpdateV2::try_deserialize(&mut account_info.data).map_err(|_| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: "Failed to parse Pyth price account".into(),
                    }
                })?;

            return Ok(AccountUpdateEvent::PriceFeed {
                price_feed: price_account.price_message,
            });
        }

        // If the account belongs to the webhook program, parse in
        if owner_pubkey.eq(&sablier_webhook_program::ID) && account_info.data.len() > 8 {
            return Ok(AccountUpdateEvent::Webhook {
                webhook: Webhook::try_deserialize(&mut account_info.data).map_err(|_| {
                    GeyserPluginError::AccountsUpdateError {
                        msg: "Failed to parse Sablier webhook".into(),
                    }
                })?,
            });
        }

        Err(GeyserPluginError::AccountsUpdateError {
            msg: "Account is not relevant to Sablier plugin".into(),
        })
    }
}
