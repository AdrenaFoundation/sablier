use anchor_lang::prelude::*;

use crate::constants::SEED_FEE;

/// Escrows the lamport balance owed to a particular worker.
#[account]
#[derive(Debug, InitSpace)]
pub struct Fee {
    /// The number of lamports that are distributable for this epoch period.
    pub distributable_balance: u64,
    /// The worker who received the fees.
    pub worker: Pubkey,
    pub bump: u8,
}

impl Fee {
    /// Derive the pubkey of a fee account.
    pub fn pubkey(worker: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_FEE, worker.as_ref()], &crate::ID).0
    }

    /// Derive the pubkey of a fee account.
    pub fn key(&self) -> Pubkey {
        Fee::pubkey(self.worker)
    }
}

/// Trait for reading and writing to a fee account.
pub trait FeeAccount {
    /// Initialize the account to hold fee object.
    fn init(&mut self, worker: Pubkey, bump: u8) -> Result<()>;
}

impl FeeAccount for Account<'_, Fee> {
    fn init(&mut self, worker: Pubkey, bump: u8) -> Result<()> {
        self.distributable_balance = 0;
        self.worker = worker;
        self.bump = bump;
        Ok(())
    }
}
