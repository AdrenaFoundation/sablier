use {
    crate::{errors::*, objects::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction(amount: u64, penalty: bool)]
pub struct FeeCollect<'info> {
    #[account(
        mut,
        seeds = [
            SEED_FEE,
            fee.worker.as_ref(),
        ],
        bump,
    )]
    pub fee: Account<'info, Fee>,

    pub signer: Signer<'info>,
}

pub fn handler(ctx: Context<FeeCollect>, amount: u64, penalty: bool) -> Result<()> {
    // Get accounts.
    let fee = &mut ctx.accounts.fee;

    // Increment the collected fee counter.
    if penalty {
        fee.penalty_balance += amount;
    } else {
        fee.collected_balance += amount;
    }

    // Verify there are enough lamports to distribute at the end of the epoch.
    let lamport_balance = fee.get_lamports();
    let data_len = 8 + fee.try_to_vec()?.len();
    let min_rent_balance = Rent::get()?.minimum_balance(data_len);

    msg!(
        "Fee collection! lamports: {} collected: {} penalty: {} rent: {}",
        lamport_balance,
        fee.collected_balance,
        fee.penalty_balance,
        min_rent_balance
    );

    require!(
        (fee.collected_balance + fee.penalty_balance + min_rent_balance) >= lamport_balance
        ClockworkError::InsufficientFeeBalance
    );

    Ok(())
}
