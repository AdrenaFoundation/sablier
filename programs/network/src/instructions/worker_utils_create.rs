use anchor_lang::prelude::*;

use crate::{
    constants::{SEED_FEE, SEED_PENALTY, SEED_REGISTRY, SEED_WORKER},
    Fee, FeeAccount, Penalty, PenaltyAccount, Registry, Worker,
};

#[derive(Accounts)]
pub struct WorkerUtilsCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [
            SEED_WORKER,
            registry.total_workers.to_be_bytes().as_ref(),
        ],
        bump = worker.bump,
    )]
    pub worker: Account<'info, Worker>,

    #[account(
        seeds = [SEED_REGISTRY],
        bump = registry.bump,
    )]
    pub registry: Account<'info, Registry>,

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

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WorkerUtilsCreate>) -> Result<()> {
    let worker = &mut ctx.accounts.worker;
    let fee = &mut ctx.accounts.fee;
    let penalty = &mut ctx.accounts.penalty;

    fee.init(worker.key(), ctx.bumps.fee)?;
    penalty.init(worker.key(), ctx.bumps.penalty)?;

    Ok(())
}
