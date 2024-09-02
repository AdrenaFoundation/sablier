use anchor_lang::{AccountDeserialize, Discriminator};
use pyth_solana_receiver_sdk::price_update::{PriceFeedMessage, PriceUpdateV2};
use sablier_thread_program::state::{Thread, VersionedThread};
use sablier_webhook_program::state::Webhook;
use solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoVersions;
use solana_sdk::{clock::Clock, pubkey::Pubkey, sysvar};

use crate::error::PluginError;

#[derive(Debug)]
pub struct AccountUpdate {
    pub key: Pubkey,
    pub event: Option<AccountUpdateEvent>,
}

#[derive(Debug)]
pub enum AccountUpdateEvent {
    Clock { clock: Clock },
    Thread { thread: VersionedThread },
    PriceFeed { price_feed: PriceFeedMessage },
    Webhook { webhook: Webhook },
}

impl<'a> From<ReplicaAccountInfoVersions<'a>> for AccountUpdate {
    fn from(value: ReplicaAccountInfoVersions<'a>) -> Self {
        let (key, owner, data) = match value {
            ReplicaAccountInfoVersions::V0_0_1(acc) => (acc.pubkey, acc.owner, acc.data),
            ReplicaAccountInfoVersions::V0_0_2(acc) => (acc.pubkey, acc.owner, acc.data),
            ReplicaAccountInfoVersions::V0_0_3(acc) => (acc.pubkey, acc.owner, acc.data),
        };

        // Parse pubkeys.
        let owner = Pubkey::try_from(owner).unwrap_or_default();
        let key = Pubkey::try_from(key).unwrap_or_default();

        let event = parse_event(key, owner, data).unwrap_or_default();

        AccountUpdate { key, event }
    }
}

fn parse_event(
    key: Pubkey,
    owner: Pubkey,
    mut data: &[u8],
) -> Result<Option<AccountUpdateEvent>, PluginError> {
    if key == sysvar::clock::ID {
        return Ok(Some(AccountUpdateEvent::Clock {
            clock: bincode::deserialize::<Clock>(data)?,
        }));
    }

    if owner == sablier_thread_program::ID && data.len() > 8 {
        let d = &data[..8];
        if d == Thread::discriminator() {
            return Ok(Some(AccountUpdateEvent::Thread {
                thread: VersionedThread::V1(Thread::try_deserialize(&mut data)?),
            }));
        }
    }

    if owner == pyth_solana_receiver_sdk::ID {
        return Ok(Some(AccountUpdateEvent::PriceFeed {
            price_feed: PriceUpdateV2::try_deserialize(&mut data)?.price_message,
        }));
    }

    if owner == sablier_webhook_program::ID && data.len() > 8 {
        return Ok(Some(AccountUpdateEvent::Webhook {
            webhook: Webhook::try_deserialize(&mut data)?,
        }));
    }

    Ok(None)
}
