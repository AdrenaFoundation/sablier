use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
};

/// Accounts required by the `thread_resume` instruction.
#[derive(Accounts)]
pub struct ThreadResume<'info> {
    /// The authority (owner) of the thread.
    pub authority: Signer<'info>,

    /// The thread to be resumed.
    #[account(
        mut,
        seeds = [
            SEED_THREAD,
            thread.authority.as_ref(),
            thread.id.as_slice(),
            thread.domain.as_ref().unwrap_or(&Vec::new()).as_slice()
        ],
        bump = thread.bump,
        has_one = authority
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler(ctx: Context<ThreadResume>) -> Result<()> {
    // Get accounts
    let thread = &mut ctx.accounts.thread;

    // Resume the thread
    thread.paused = false;

    // Update the exec context
    if let Some(exec_context) = thread.exec_context {
        if let TriggerContext::Cron { started_at: _ } = exec_context.trigger_context {
            // Jump ahead to the current timestamp
            thread.exec_context = Some(ExecContext {
                trigger_context: TriggerContext::Cron {
                    started_at: Clock::get()?.unix_timestamp,
                },
                ..exec_context
            });
        }
    }

    Ok(())
}
