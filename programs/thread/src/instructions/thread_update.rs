use crate::{constants::*, errors::SablierError, state::*};

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use sablier_utils::account::AccountInfoExt;

/// Accounts required by the `thread_update` instruction.
#[derive(Accounts)]
#[instruction(settings: ThreadSettings)]
pub struct ThreadUpdate<'info> {
    /// The authority (owner) of the thread.
    pub authority: Signer<'info>,

    /// The payer of the reallocation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Solana system program
    pub system_program: Program<'info, System>,

    /// The thread to be updated.
    #[account(
            mut,
            seeds = [
                SEED_THREAD,
                thread.authority.as_ref(),
                thread.id.as_slice(),
                thread.domain.as_ref().unwrap_or(&Vec::new()).as_slice()
            ],
            bump = thread.bump,
            has_one = authority,
        )]
    pub thread: Account<'info, Thread>,
}

pub fn handler(ctx: Context<ThreadUpdate>, settings: ThreadSettings) -> Result<()> {
    // Get accounts
    let payer = &ctx.accounts.payer;
    let thread = &mut ctx.accounts.thread;
    let system_program = &ctx.accounts.system_program;

    // Update the thread.
    if let Some(fee) = settings.fee {
        thread.fee = fee;
    }

    // If provided, update the thread's instruction set.
    if let Some(instructions) = settings.instructions {
        thread.instructions = instructions;
    }

    // If provided, update the rate limit.
    if let Some(rate_limit) = settings.rate_limit {
        thread.rate_limit = rate_limit;
    }

    // If provided, update the thread's trigger and reset the exec context.
    if let Some(trigger) = settings.trigger {
        // Require the thread is not in the middle of processing.
        require!(
            std::mem::discriminant(&thread.trigger) == std::mem::discriminant(&trigger),
            SablierError::InvalidTriggerVariant
        );
        thread.trigger = trigger.clone();

        // If the user updates an account trigger, the trigger context is no longer valid.
        // Here we reset the trigger context to zero to re-prime the trigger.
        if thread.exec_context.is_some() {
            thread.exec_context = Some(ExecContext {
                trigger_context: match trigger {
                    Trigger::Account {
                        address: _,
                        offset: _,
                        size: _,
                    } => TriggerContext::Account { data_hash: 0 },
                    _ => thread.exec_context.unwrap().trigger_context,
                },
                ..thread.exec_context.unwrap()
            });
        }
    }

    // Reallocate mem for the thread account
    thread.realloc_account()?;

    // If lamports are required to maintain rent-exemption, pay them
    let data_len = thread.data_len();
    let minimum_rent = Rent::get()?.minimum_balance(data_len);
    let thread_lamports = thread.get_lamports();
    if minimum_rent > thread_lamports {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: thread.to_account_info(),
                },
            ),
            minimum_rent - thread_lamports,
        )?;
    }

    Ok(())
}
