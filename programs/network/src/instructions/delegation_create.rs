use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct DelegationCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = Config::pubkey())]
    pub config: Account<'info, Config>,

    #[account(
        init,
        seeds = [
            SEED_DELEGATION,
            worker.key().as_ref(),
            worker.total_delegations.to_be_bytes().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Delegation::INIT_SPACE,
    )]
    pub delegation: Account<'info, Delegation>,

    #[account(
        init,
        payer = authority,
        associated_token::authority = delegation,
        associated_token::mint = mint,
    )]
    pub delegation_tokens: Account<'info, TokenAccount>,

    #[account(address = config.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            SEED_WORKER,
            worker.id.to_be_bytes().as_ref(),
        ],
        bump
    )]
    pub worker: Account<'info, Worker>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DelegationCreate>) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let delegation = &mut ctx.accounts.delegation;
    let worker = &mut ctx.accounts.worker;

    // Initialize the delegation account.
    delegation.init(authority.key(), worker.total_delegations, worker.key())?;

    // Increment the worker's total delegations counter.
    worker.total_delegations += 1;

    Ok(())
}
