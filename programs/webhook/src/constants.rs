use anchor_lang::prelude::*;

#[constant]
pub const SEED_WEBHOOK: &[u8] = b"webhook";

#[constant]
pub const WEBHOOK_FEE: u64 = 1_000_000;

#[constant]
pub const TIMEOUT_THRESHOLD: u64 = 100; // 100 slots
