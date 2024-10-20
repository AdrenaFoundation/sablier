use {crate::state::*, anchor_lang::prelude::*, sablier_network_program::state::Config};
/// Accounts required by the `thread_reset` instruction.
#[derive(Accounts)]
pub struct ThreadDeleteBypass<'info> {
    #[account(has_one = admin)]
    pub config: AccountLoader<'info, Config>,
    pub admin: Signer<'info>,
    ///CHECKS
    #[account(mut)]
    pub close_to: UncheckedAccount<'info>,
    /// The thread to be paused.
    #[account(mut, close = close_to)]
    pub thread: Account<'info, Thread>,
}

pub fn handler(_ctx: Context<ThreadDeleteBypass>) -> Result<()> {
    Ok(())
}
