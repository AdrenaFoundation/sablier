use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
};

/// Accounts required by the `thread_withdraw` instruction.
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ThreadWithdraw<'info> {
    /// The authority (owner) of the thread.
    pub authority: Signer<'info>,

    /// The account to withdraw lamports to.
    #[account(mut)]
    pub pay_to: SystemAccount<'info>,

    /// The thread to be.
    #[account(
        mut,
        seeds = [
            SEED_THREAD,
            thread.authority.as_ref(),
            thread.id.as_slice(),
        ],
        bump = thread.bump,
        has_one = authority,
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler(ctx: Context<ThreadWithdraw>, amount: u64) -> Result<()> {
    // Get accounts
    let pay_to = &mut ctx.accounts.pay_to;
    let thread = &mut ctx.accounts.thread;

    // Calculate the minimum rent threshold
    let data_len = 8 + thread.try_to_vec()?.len();
    let minimum_rent = Rent::get()?.minimum_balance(data_len);
    let post_balance = thread.get_lamports() - amount;

    require!(
        post_balance > minimum_rent,
        SablierError::WithdrawalTooLarge
    );

    // Withdraw balance from thread to the pay_to account
    thread.sub_lamports(amount)?;
    pay_to.add_lamports(amount)?;

    Ok(())
}
