use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        seeds = [SEED_CONFIG],
        bump,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [SEED_REGISTRY],
        bump,
        payer = admin,
        space = 8 + Registry::INIT_SPACE,
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        init,
        seeds = [
            SEED_SNAPSHOT,
            0_u64.to_be_bytes().as_ref(),
        ],
        bump,
        payer = admin,
        space = 8 + Snapshot::INIT_SPACE,
    )]
    pub snapshot: Account<'info, Snapshot>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    // Get accounts
    let admin = &ctx.accounts.admin;
    let config = &mut ctx.accounts.config;
    let mint = &ctx.accounts.mint;
    let registry = &mut ctx.accounts.registry;
    let snapshot = &mut ctx.accounts.snapshot;

    // Initialize accounts.
    config.init(admin.key(), mint.key())?;
    registry.init()?;
    snapshot.init(0)?;

    Ok(())
}
