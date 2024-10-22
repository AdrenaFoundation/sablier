use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{transfer, Mint, Token, TokenAccount, Transfer},
    },
};

#[derive(Accounts)]
pub struct DelegationFix<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        mut,
        seeds = [
            SEED_DELEGATION,
            worker.key().as_ref(),
            delegation_0.id.to_be_bytes().as_ref()
        ],
        bump,
        has_one = authority,
    )]
    pub delegation_0: Account<'info, Delegation>,

    #[account(
        init,
        payer = authority,
        associated_token::authority = delegation_0,
        associated_token::mint = mint,
    )]
    pub delegation_tokens_0: Account<'info, TokenAccount>,

    #[account(address = config.load()?.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        close = authority,
        seeds = [
            SEED_DELEGATION,
            worker.key().as_ref(),
            delegation_1.id.to_be_bytes().as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub delegation_1: Account<'info, Delegation>,

    #[account(
        mut,
        close = authority,
        associated_token::authority = delegation_1,
        associated_token::mint = mint,
    )]
    pub delegation_tokens_1: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            SEED_WORKER,
            worker.id.to_be_bytes().as_ref(),
        ],
        bump,
    )]
    pub worker: Account<'info, Worker>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DelegationFix>) -> Result<()> {
    // Get accounts.
    let delegation_1 = &ctx.accounts.delegation_1;
    let delegation_tokens_0 = &ctx.accounts.delegation_tokens_0;
    let delegation_tokens_1 = &ctx.accounts.delegation_tokens_1;
    let token_program = &ctx.accounts.token_program;
    let worker = &mut ctx.accounts.worker;

    // Transfer tokens from authority tokens to delegation
    let bump = ctx.bumps.delegation_1;
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: delegation_tokens_1.to_account_info(),
                to: delegation_tokens_0.to_account_info(),
                authority: delegation_1.to_account_info(),
            },
            &[&[
                SEED_DELEGATION,
                delegation_1.worker.as_ref(),
                delegation_1.id.to_be_bytes().as_ref(),
                &[bump],
            ]],
        ),
        delegation_tokens_1.amount,
    )?;

    worker.total_delegations = 1;

    Ok(())
}
