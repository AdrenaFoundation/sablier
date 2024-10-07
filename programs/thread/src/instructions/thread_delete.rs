use {
    crate::{constants::SEED_THREAD, state::*},
    anchor_lang::{prelude::*, solana_program::system_program},
    sablier_network_program::errors::SablierError,
};

/// Accounts required by the `thread_delete` instruction.
#[derive(Accounts)]
pub struct ThreadDelete<'info> {
    /// The authority (owner) of the thread.
    #[account(
        // constraint = authority.key().eq(&thread.authority) || authority.key().eq(&thread.key())
    )]
    pub authority: Signer<'info>,

    /// The address to return the data rent lamports to.
    #[account(mut)]
    pub close_to: SystemAccount<'info>,

    /// The thread to be deleted.
    #[account(mut)]
    pub thread: UncheckedAccount<'info>,
    // #[account(
    //     mut,
    //     seeds = [
    //         SEED_THREAD,
    //         thread.authority.as_ref(),
    //         thread.id.as_slice(),
    //         thread.domain.as_ref().unwrap_or(&Vec::new()).as_slice()
    //     ],
    //     bump = thread.bump,
    // )]
    // pub thread: Account<'info, Thread>,
}

pub fn handler(ctx: Context<ThreadDelete>) -> Result<()> {
    let thread = &ctx.accounts.thread;
    let close_to = &ctx.accounts.close_to;

    // We want this instruction not to fail if the thread is already deleted or inexistent.
    // As such, all checks are done in the code that than in anchor (see commented code above)
    // First, must try to deserialize the thread.

    // Get either V1 or V2 thread - If the provided thread does not exist, print an error message and return Ok.
    let thread = match Thread::try_deserialize_unchecked(&mut thread.data.borrow_mut().as_ref()) {
        Ok(t) => t,
        Err(_) => {
            msg!("Not a thread or account does not exist");
            return Ok(());
        }
    };

    // Preliminary checks
    {
        // Verify the authority
        let authority_key = ctx.accounts.authority.key;
        let thread_key = ctx.accounts.thread.key;

        require!(
            thread.authority.eq(authority_key) || authority_key.eq(thread_key),
            SablierError::InvalidThreadAuthority
        );

        // Verify the account provided
        let thread_account = &ctx.accounts.thread;
        {
            // Verify the account is initialized
            require!(
                thread_account.owner != &system_program::ID && thread_account.lamports() > 0,
                SablierError::InvalidThreadAccount
            );

            // Verify the account is owned by the program
            require!(
                thread_account.owner == &crate::ID,
                SablierError::InvalidThreadAccount
            );

            // Verify the seed derivation
            let default_vec = Vec::new();
            let thread_bump = thread.bump.to_le_bytes();
            let seed = [
                SEED_THREAD,
                thread.authority.as_ref(),
                thread.id.as_slice(),
                thread.domain.as_ref().unwrap_or(&default_vec).as_slice(),
                thread_bump.as_ref(),
            ];
            let expected_thread_key = Pubkey::create_program_address(&seed, &crate::ID)
                .map_err(|_| SablierError::InvalidThreadAccount)?;
            require!(
                expected_thread_key == *thread_key,
                SablierError::InvalidThreadAccount
            );
        }
    }

    // Transfer lamports out (implicit close)
    {
        let thread_account = &ctx.accounts.thread;
        let thread_lamports = thread_account.get_lamports();
        thread_account.sub_lamports(thread_lamports)?;
        close_to.add_lamports(thread_lamports)?;
    }
    Ok(())
}
