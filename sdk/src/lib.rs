#[cfg(test)]
mod litesvm;

pub use sablier_thread_program::errors;
pub use sablier_thread_program::program::ThreadProgram;
pub use sablier_thread_program::ID;

pub mod state {
    pub use sablier_thread_program::state::{
        ClockData, ExecContext, SerializableAccount, SerializableInstruction, Thread,
        ThreadAccount, ThreadResponse, ThreadSettings, Trigger, TriggerContext,
    };
}

pub mod utils {
    pub use sablier_thread_program::state::Equality;
    pub use sablier_thread_program::state::PAYER_PUBKEY;
}

pub mod cpi {
    use anchor_lang::prelude::{CpiContext, Result};

    pub use sablier_thread_program::cpi::accounts::{
        ThreadCreate, ThreadDelete, ThreadPause, ThreadReset, ThreadResume, ThreadUpdate,
        ThreadWithdraw,
    };

    pub fn thread_create<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadCreate<'info>>,
        amount: u64,
        id: Vec<u8>,
        domain: Option<Vec<u8>>,
        instructions: Vec<crate::state::SerializableInstruction>,
        trigger: crate::state::Trigger,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_create(ctx, amount, id, domain, instructions, trigger)
    }

    pub fn thread_delete<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadDelete<'info>>,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_delete(ctx)
    }

    pub fn thread_pause<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadPause<'info>>,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_pause(ctx)
    }

    pub fn thread_resume<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadResume<'info>>,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_resume(ctx)
    }

    pub fn thread_reset<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadReset<'info>>,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_reset(ctx)
    }

    pub fn thread_update<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadUpdate<'info>>,
        settings: crate::state::ThreadSettings,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_update(ctx, settings)
    }

    pub fn thread_withdraw<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, ThreadWithdraw<'info>>,
        amount: u64,
    ) -> Result<()> {
        sablier_thread_program::cpi::thread_withdraw(ctx, amount)
    }
}
