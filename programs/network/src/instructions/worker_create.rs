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

    #[account(
        init,
        seeds = [
            SEED_FEE,
            worker.key().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Fee::INIT_SPACE,
    )]
    pub fee: Account<'info, Fee>,

    #[account(
        init,
        seeds = [
            SEED_PENALTY,
            worker.key().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Penalty::INIT_SPACE,
    )]
    pub penalty: Account<'info, Penalty>,

    #[account(address = config.load()?.mint)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
        constraint = !registry.locked @ SablierError::RegistryLocked
    )]
    pub registry: Account<'info, Registry>,

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
    pub worker: Account<'info, Worker>,

    #[account(
        init,
        payer = authority,
        associated_token::authority = worker,
        associated_token::mint = mint,
    )]
    pub worker_tokens: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WorkerCreate>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let fee = &mut ctx.accounts.fee;
    let penalty = &mut ctx.accounts.penalty;
    let registry = &mut ctx.accounts.registry;
    let signatory = &mut ctx.accounts.signatory;
    let worker = &mut ctx.accounts.worker;

    // Initialize the worker accounts.
    worker.init(authority, registry.total_workers, signatory)?;
    fee.init(worker.key())?;
    penalty.init(worker.key())?;

    // Update the registry's worker counter.
    registry.total_workers += 1;

    Ok(())
}
