use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::constants::SEED_PENALTY;

/// Escrows the lamport balance owed to a particular worker.
#[account]
#[derive(Debug, InitSpace)]
pub struct Penalty {
    /// The worker who was penalized.
    pub worker: Pubkey,
    pub bump: u8,
}

impl Penalty {
    /// Derive the pubkey of a fee account.
    pub fn pubkey(worker: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_PENALTY, worker.as_ref()], &crate::ID).0
    }
}

/// Trait for reading and writing to a penalty account.
pub trait PenaltyAccount {
    /// Get the pubkey of the penalty account.
    fn pubkey(&self) -> Pubkey;

    /// Initialize the account to hold penalty object.
    fn init(&mut self, worker: Pubkey, bump: u8) -> Result<()>;
}

impl PenaltyAccount for Account<'_, Penalty> {
    fn pubkey(&self) -> Pubkey {
        Penalty::pubkey(self.worker)
    }

    fn init(&mut self, worker: Pubkey, bump: u8) -> Result<()> {
        self.worker = worker;
        self.bump = bump;
        Ok(())
    }
}
