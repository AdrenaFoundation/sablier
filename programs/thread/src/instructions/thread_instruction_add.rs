use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use sablier_utils::account::AccountInfoExt;

use crate::{constants::*, state::*};

/// Accounts required by the `thread_instruction_add` instruction.
#[derive(Accounts)]
#[instruction(instruction: SerializableInstruction)]
pub struct ThreadInstructionAdd<'info> {
    /// The authority (owner) of the thread.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The Solana system program.
    pub system_program: Program<'info, System>,

    /// The thread to be paused.
    #[account(
        mut,
        seeds = [
            SEED_THREAD,
            thread.authority.as_ref(),
            thread.id.as_slice(),
        ],
        bump = thread.bump,
        has_one = authority
    )]
    pub thread: Account<'info, Thread>,
}

pub fn handler(
    ctx: Context<ThreadInstructionAdd>,
    instruction: SerializableInstruction,
) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let thread = &mut ctx.accounts.thread;
    let system_program = &ctx.accounts.system_program;

    // Append the instruction.
    thread.instructions.push(instruction);

    // Reallocate mem for the thread account.
    thread.realloc_account()?;

    // If lamports are required to maintain rent-exemption, pay them.
    let data_len = thread.data_len();
    let minimum_rent = Rent::get()?.minimum_balance(data_len);
    let thread_lamports = thread.get_lamports();
    if minimum_rent > thread_lamports {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: authority.to_account_info(),
                    to: thread.to_account_info(),
                },
            ),
            minimum_rent - thread_lamports,
        )?;
    }

    Ok(())
}
