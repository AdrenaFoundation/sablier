use anchor_lang::prelude::*;
use solana_program::{clock::Clock, pubkey as key, pubkey::Pubkey};

pub const PYTH_PUSH_ORACLE_ID: Pubkey = key!("pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT");
pub const ID: Pubkey = key!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");

pub type FeedId = [u8; 32];

#[error_code]
#[derive(PartialEq)]
pub enum GetPriceError {
    #[msg("This price feed update's age exceeds the requested maximum age")]
    PriceTooOld = 10000, // Big number to avoid conflicts with the SDK user's program error codes
    #[msg("The price feed update doesn't match the requested feed id")]
    MismatchedFeedId,
    #[msg("This price feed update has a lower verification level than the one requested")]
    InsufficientVerificationLevel,
    #[msg("Feed id must be 32 Bytes, that's 64 hex characters or 66 with a 0x prefix")]
    FeedIdMustBe32Bytes,
    #[msg("Feed id contains non-hex characters")]
    FeedIdNonHexCharacter,
}

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct PriceFeedMessage {
    pub feed_id: FeedId,
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
    pub prev_publish_time: i64,
    pub ema_price: i64,
    pub ema_conf: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum VerificationLevel {
    Partial { num_signatures: u8 },
    Full,
}

impl VerificationLevel {
    pub fn gte(&self, other: VerificationLevel) -> bool {
        match self {
            VerificationLevel::Full => true,
            VerificationLevel::Partial { num_signatures } => match other {
                VerificationLevel::Full => false,
                VerificationLevel::Partial {
                    num_signatures: other_num_signatures,
                } => *num_signatures >= other_num_signatures,
            },
        }
    }
}

pub struct Price {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PriceUpdateV2 {
    pub write_authority: Pubkey,
    pub verification_level: VerificationLevel,
    pub price_message: PriceFeedMessage,
    pub posted_slot: u64,
}

impl PriceUpdateV2 {
    pub fn get_price_unchecked(
        &self,
        feed_id: &FeedId,
    ) -> std::result::Result<Price, GetPriceError> {
        check!(
            self.price_message.feed_id == *feed_id,
            GetPriceError::MismatchedFeedId
        );
        Ok(Price {
            price: self.price_message.price,
            conf: self.price_message.conf,
            exponent: self.price_message.exponent,
            publish_time: self.price_message.publish_time,
        })
    }

    pub fn get_price_no_older_than_with_custom_verification_level(
        &self,
        clock: &Clock,
        maximum_age: u64,
        feed_id: &FeedId,
        verification_level: VerificationLevel,
    ) -> std::result::Result<Price, GetPriceError> {
        check!(
            self.verification_level.gte(verification_level),
            GetPriceError::InsufficientVerificationLevel
        );
        let price = self.get_price_unchecked(feed_id)?;
        check!(
            price
                .publish_time
                .saturating_add(maximum_age.try_into().unwrap())
                >= clock.unix_timestamp,
            GetPriceError::PriceTooOld
        );
        Ok(price)
    }

    pub fn get_price_no_older_than(
        &self,
        clock: &Clock,
        maximum_age: u64,
        feed_id: &FeedId,
    ) -> std::result::Result<Price, GetPriceError> {
        self.get_price_no_older_than_with_custom_verification_level(
            clock,
            maximum_age,
            feed_id,
            VerificationLevel::Full,
        )
    }
}

impl anchor_lang::AccountDeserialize for PriceUpdateV2 {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        if buf.len() < 8 {
            return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound.into());
        }
        let given_disc = &buf[..8];
        if [34, 241, 35, 99, 157, 126, 244, 205] != given_disc {
            return Err(anchor_lang::error::Error::from(
                anchor_lang::error::AnchorError {
                    error_name: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch.name(),
                    error_code_number: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                        .into(),
                    error_msg: anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch
                        .to_string(),
                    error_origin: Some(anchor_lang::error::ErrorOrigin::AccountName(
                        "PriceUpdateV2".to_string(),
                    )),
                    compared_values: None,
                },
            ));
        }
        Self::try_deserialize_unchecked(buf)
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let mut data: &[u8] = &buf[8..];
        AnchorDeserialize::deserialize(&mut data)
            .map_err(|_| anchor_lang::error::ErrorCode::AccountDidNotDeserialize.into())
    }
}

pub fn get_oracle_key(shard_id: u16, feed_id: FeedId) -> Pubkey {
    let (pubkey, _) =
        Pubkey::find_program_address(&[&shard_id.to_be_bytes(), &feed_id], &PYTH_PUSH_ORACLE_ID);
    pubkey
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::*;

    #[test]
    fn test_price_update_v2() {
        let data = base64::engine::general_purpose::STANDARD.decode("IvEjY51+9M1gMUcENA3t3zcf1CRyFI8kjp0abRpesqw6zYt/1dayQwHvDYtv2izrpB2hXUCV0do5Kg0vjtDGx7wPTPrIwoC1bdYkWB4DAAAAPA/bAAAAAAD4////rbTWZgAAAACttNZmAAAAAOx1oSEDAAAAtpvRAAAAAADMB0YTAAAAAAA=").unwrap();

        let price_update = PriceUpdateV2::try_deserialize(&mut data.as_slice()).unwrap();

        assert_eq!(
            price_update.write_authority,
            key!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE")
        );
    }
}
