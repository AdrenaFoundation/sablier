use anchor_lang::prelude::*;

#[constant]
pub const SEED_THREAD: &[u8] = b"thread";

/// The minimum exec fee that may be set on a thread.
#[constant]
pub const THREAD_MINIMUM_FEE: u64 = 1000;

/// Static space for next_instruction field.
#[constant]
pub const NEXT_INSTRUCTION_SIZE: usize = 1232;

/// The ID of the pool workers must be a member of to collect fees.
#[constant]
pub const POOL_ID: u64 = 0;

/// The number of lamports to reimburse the worker with after they've submitted a transaction's worth of exec instructions.
#[constant]
pub const TRANSACTION_BASE_FEE_REIMBURSEMENT: u64 = 5_000;
