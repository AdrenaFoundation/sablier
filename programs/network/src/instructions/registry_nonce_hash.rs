use sablier_utils::thread::ThreadResponse;

use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct RegistryNonceHash<'info> {
    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump
    )]
    pub registry: Account<'info, Registry>,

    #[account(address = config.load()?.hasher_thread)]
    pub thread: Signer<'info>,
}

pub fn handler(ctx: Context<RegistryNonceHash>) -> Result<ThreadResponse> {
    let registry = &mut ctx.accounts.registry;
    registry.hash_nonce()?;
    Ok(ThreadResponse::default())
}
