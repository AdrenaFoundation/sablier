use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::constants::SEED_CONFIG;

/**
 * Config
 */

#[account(zero_copy)]
#[derive(Debug, InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub epoch_thread: Pubkey,
    pub hasher_thread: Pubkey,
    pub mint: Pubkey,
}

impl Config {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[SEED_CONFIG], &crate::ID).0
    }
}

/**
 * ConfigSettings
 */

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigSettings {
    pub admin: Pubkey,
    pub epoch_thread: Pubkey,
    pub hasher_thread: Pubkey,
    pub mint: Pubkey,
}

/**
 * ConfigAccount
 */

pub trait ConfigAccount {
    fn init(&mut self, admin: Pubkey, mint: Pubkey) -> Result<()>;

    fn update(&mut self, settings: ConfigSettings) -> Result<()>;
}

impl ConfigAccount for AccountLoader<'_, Config> {
    fn init(&mut self, admin: Pubkey, mint: Pubkey) -> Result<()> {
        let mut config = self.load_init()?;
        config.admin = admin;
        config.mint = mint;
        Ok(())
    }

    fn update(&mut self, settings: ConfigSettings) -> Result<()> {
        let mut config = self.load_mut()?;
        config.admin = settings.admin;
        config.epoch_thread = settings.epoch_thread;
        config.hasher_thread = settings.hasher_thread;
        config.mint = settings.mint;
        Ok(())
    }
}
