use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct PenaltyClaim<'info> {
    #[account(address = config.load()?.admin)]
    pub admin: Signer<'info>,

    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(mut)]
    pub pay_to: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PENALTY,
            penalty.worker.as_ref(),
        ],
        bump,
    )]
    pub penalty: Account<'info, Penalty>,
}

pub fn handler(ctx: Context<PenaltyClaim>) -> Result<()> {
    // Get accounts.
    let penalty = &mut ctx.accounts.penalty;
    let pay_to = &mut ctx.accounts.pay_to;

    // Calculate how  many lamports are
    let lamport_balance = penalty.get_lamports();
    let data_len = 8 + Penalty::INIT_SPACE;
    let min_rent_balance = Rent::get()?.minimum_balance(data_len);
    let claimable_balance = lamport_balance - min_rent_balance;
    require!(
        claimable_balance > 0,
        SablierError::InsufficientPenaltyBalance
    );

    // Pay reimbursment for base transaction fee
    penalty.sub_lamports(claimable_balance)?;
    pay_to.add_lamports(claimable_balance)?;

    Ok(())
}
