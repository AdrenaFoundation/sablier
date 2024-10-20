use {crate::state::*, anchor_lang::prelude::*, sablier_network_program::state::Config};
/// Accounts required by the `thread_reset` instruction.
#[derive(Accounts)]
pub struct ThreadResetNext<'info> {
    #[account(has_one = admin)]
    pub config: AccountLoader<'info, Config>,
    pub admin: Signer<'info>,
    /// The thread to be paused.
    #[account(mut)]
    pub thread: Account<'info, Thread>,
}
pub fn handler(ctx: Context<ThreadResetNext>, timestamp: i64) -> Result<()> {
    // Get accounts
    let thread = &mut ctx.accounts.thread;
    let clock = Clock::get()?;
    // Full reset the thread state.
    thread.exec_context = Some(ExecContext {
        exec_index: 0,
        execs_since_reimbursement: 0,
        execs_since_slot: 0,
        last_exec_at: clock.slot,
        trigger_context: TriggerContext::Periodic {
            started_at: timestamp,
        },
    });
    thread.next_instruction = None;
    Ok(())
}
