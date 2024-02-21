use anchor_lang::prelude::*;
use sablier_utils::thread::ThreadResponse;

use crate::{constants::*, state::*};

#[derive(Accounts)]
pub struct EpochCutover<'info> {
    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
    )]
    pub registry: Account<'info, Registry>,

    #[account(address = config.load()?.epoch_thread)]
    pub thread: Signer<'info>,
}

pub fn handler(ctx: Context<EpochCutover>) -> Result<ThreadResponse> {
    let registry = &mut ctx.accounts.registry;
    registry.current_epoch += 1;
    registry.locked = false;

    Ok(ThreadResponse {
        close_to: None,
        dynamic_instruction: None,
        trigger: None,
    })
}
