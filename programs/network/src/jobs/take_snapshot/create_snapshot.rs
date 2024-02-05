use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_spl::associated_token::get_associated_token_address;
use sablier_utils::thread::{ThreadResponse, PAYER_PUBKEY};

use crate::{constants::*, state::*};

#[derive(Accounts)]
pub struct TakeSnapshotCreateSnapshot<'info> {
    #[account(address = Config::pubkey())]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        address = Registry::pubkey(),
        constraint = registry.locked
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        init,
        seeds = [
            SEED_SNAPSHOT,
            (registry.current_epoch + 1).to_be_bytes().as_ref(),
        ],
        bump,
        space = 8 + Snapshot::INIT_SPACE,
        payer = payer
    )]
    pub snapshot: Account<'info, Snapshot>,

    pub system_program: Program<'info, System>,

    #[account(address = config.epoch_thread)]
    pub thread: Signer<'info>,
}

pub fn handler(ctx: Context<TakeSnapshotCreateSnapshot>) -> Result<ThreadResponse> {
    // Get accounts
    let config = &ctx.accounts.config;
    let registry = &ctx.accounts.registry;
    let snapshot = &mut ctx.accounts.snapshot;
    let system_program = &ctx.accounts.system_program;
    let thread = &ctx.accounts.thread;

    // Start a new snapshot.
    snapshot.init(registry.current_epoch + 1)?;

    Ok(ThreadResponse {
        dynamic_instruction: if registry.total_workers > 0 {
            // The registry has workers. Create a snapshot frame for the zeroth worker.
            let snapshot_frame_pubkey = SnapshotFrame::pubkey(snapshot.key(), 0);
            let worker_pubkey = Worker::pubkey(0);
            Some(
                Instruction {
                    program_id: crate::ID,
                    accounts: crate::accounts::TakeSnapshotCreateFrame {
                        config: config.key(),
                        payer: PAYER_PUBKEY,
                        registry: registry.key(),
                        snapshot: snapshot.key(),
                        snapshot_frame: snapshot_frame_pubkey,
                        system_program: system_program.key(),
                        thread: thread.key(),
                        worker: worker_pubkey,
                        worker_stake: get_associated_token_address(&worker_pubkey, &config.mint),
                    }
                    .to_account_metas(Some(true)),
                    data: crate::instruction::TakeSnapshotCreateFrame {}.data(),
                }
                .into(),
            )
        } else {
            None
        },
        close_to: None,
        trigger: None,
    })
}
