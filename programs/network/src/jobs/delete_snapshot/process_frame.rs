use anchor_lang::{prelude::*, InstructionData, solana_program::instruction::Instruction};
use clockwork_utils::thread::ThreadResponse;

use crate::state::*;

#[derive(Accounts)]
pub struct DeleteSnapshotProcessFrame<'info> {
    #[account(address = Config::pubkey())]
    pub config: Account<'info, Config>,

    #[account(
        address = Registry::pubkey(),
        constraint = !registry.locked
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        mut,
        seeds = [
            SEED_SNAPSHOT,
            snapshot.id.to_be_bytes().as_ref(),
        ],
        bump,
        constraint = snapshot.id < registry.current_epoch
    )]
    pub snapshot: Account<'info, Snapshot>,

    #[account(
        mut,
        seeds = [
            SEED_SNAPSHOT_FRAME,
            snapshot_frame.snapshot.as_ref(),
            snapshot_frame.id.to_be_bytes().as_ref(),
        ],
        bump,
        has_one = snapshot,
    )]
    pub snapshot_frame: Account<'info, SnapshotFrame>,

    #[account(
        mut, 
        address = config.epoch_thread
    )]
    pub thread: Signer<'info>,
}

pub fn handler(ctx: Context<DeleteSnapshotProcessFrame>) -> Result<ThreadResponse> {
    // Get accounts
    let config = &ctx.accounts.config;
    let registry = &ctx.accounts.registry;
    let snapshot = &mut ctx.accounts.snapshot;
    let snapshot_frame = &mut ctx.accounts.snapshot_frame;
    let thread = &mut ctx.accounts.thread;

    // If this frame has no entries, then close the frame account.
    if snapshot_frame.total_entries == 0 {
        let snapshot_frame_lamports = snapshot_frame.get_lamports();
        snapshot_frame.sub_lamports(snapshot_frame_lamports)?;
        thread.add_lamports(snapshot_frame_lamports)?;


        // If this is also the last frame in the snapshot, then close the snapshot account.
        if (snapshot_frame.id + 1) == snapshot.total_frames {
            let snapshot_lamports = snapshot.get_lamports();
            snapshot_frame.sub_lamports(snapshot_lamports)?;
            thread.add_lamports(snapshot_lamports)?;
        }
    }

    // Build the next instruction.
    let dynamic_instruction = if snapshot_frame.total_entries > 0 {
        // This frame has entries. Delete the entries.
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DeleteSnapshotProcessEntry {
                    config: config.key(),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_entry: SnapshotEntry::pubkey(snapshot_frame.key(), 0),
                    snapshot_frame: snapshot_frame.key(),
                    thread: thread.key(),
                }.to_account_metas(Some(true)),
                data: crate::instruction::DeleteSnapshotProcessEntry{}.data()
            }.into()
        )
    } else if (snapshot_frame.id + 1) < snapshot.total_frames {
        // There are no more entries in this frame. Move on to the next frame.
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DeleteSnapshotProcessFrame {
                    config: config.key(),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_frame: SnapshotFrame::pubkey(snapshot.key(), snapshot_frame.id + 1),
                    thread: thread.key(),
                }.to_account_metas(Some(true)),
                data: crate::instruction::DeleteSnapshotProcessFrame {}.data()
            }.into()
        )
    } else {
        // This frame has no entries, and it was the last frame. We are done!
        None
    };

    Ok( ThreadResponse { dynamic_instruction, ..ThreadResponse::default() } )
}
