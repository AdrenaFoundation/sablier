use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct WorkerCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(address = config.load()?.mint)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
        constraint = !registry.locked @ SablierError::RegistryLocked
    )]
    pub registry: Box<Account<'info, Registry>>,

    #[account(constraint = signatory.key() != authority.key() @ SablierError::InvalidSignatory)]
    pub signatory: Signer<'info>,

    #[account(
        init,
        seeds = [
            SEED_WORKER,
            registry.total_workers.to_be_bytes().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Worker::INIT_SPACE,
    )]
    pub worker: Box<Account<'info, Worker>>,

    #[account(
        init,
        payer = authority,
        associated_token::authority = worker,
        associated_token::mint = mint,
    )]
    pub worker_tokens: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WorkerCreate>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let registry = &mut ctx.accounts.registry;
    let worker = &mut ctx.accounts.worker;
    let signatory = &mut ctx.accounts.signatory;

    // Initialize the worker accounts.
    worker.init(
        authority,
        registry.total_workers,
        signatory,
        ctx.bumps.worker,
    )?;

    // Update the registry's worker counter.
    registry.total_workers += 1;

    Ok(())
}
