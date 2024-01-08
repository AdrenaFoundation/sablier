use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct WorkerClaim<'info> {
    pub authority: Signer<'info>,

    #[account(mut)]
    pub pay_to: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_WORKER,
            worker.id.to_be_bytes().as_ref()
        ],
        bump,
        has_one = authority
    )]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<WorkerClaim>, amount: u64) -> Result<()> {
    // Get accounts
    let pay_to = &mut ctx.accounts.pay_to;
    let worker = &mut ctx.accounts.worker;

    // Decrement the worker's commission balance.
    worker.commission_balance -= amount;

    // Transfer commission to the worker.
    worker.sub_lamports(amount)?;
    pay_to.add_lamports(amount)?;

    Ok(())
}
