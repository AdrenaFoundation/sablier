use pyth_solana_receiver_sdk::price_update::FeedId;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};

pub fn get_oracle_key(shard_id: u16, feed_id: FeedId) -> Pubkey {
    let (pubkey, _) = Pubkey::find_program_address(
        &[&shard_id.to_be_bytes(), &feed_id],
        &pyth_solana_receiver_sdk::PYTH_PUSH_ORACLE_ID,
    );
    pubkey
}

pub fn read_or_new_keypair(keypath: Option<String>) -> Keypair {
    match keypath {
        Some(keypath) => read_keypair_file(keypath).unwrap(),
        None => Keypair::new(),
    }
}
