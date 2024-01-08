use anchor_lang::{prelude::*, InstructionData, solana_program::instruction::Instruction};
use clockwork_utils::thread::ThreadResponse;

use crate::{state::*, constants::*};

#[derive(Accounts)]
pub struct DeleteSnapshotProcessEntry<'info> {
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
            SEED_SNAPSHOT_ENTRY,
            snapshot_entry.snapshot_frame.as_ref(),
            snapshot_entry.id.to_be_bytes().as_ref(),
        ],
        bump,
        has_one = snapshot_frame
    )]
    pub snapshot_entry: Account<'info, SnapshotEntry>,

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

pub fn handler(ctx: Context<DeleteSnapshotProcessEntry>) -> Result<ThreadResponse> {
    // Get accounts
    let config = &ctx.accounts.config;
    let registry = &ctx.accounts.registry;
    let snapshot = &mut ctx.accounts.snapshot;
    let snapshot_entry = &mut ctx.accounts.snapshot_entry;
    let snapshot_frame = &mut ctx.accounts.snapshot_frame;
    let thread = &mut ctx.accounts.thread;

    // Close the snapshot entry account.
    let snapshot_entry_lamports = snapshot_entry.get_lamports();
    snapshot_entry.sub_lamports(snapshot_entry_lamports)?;
    thread.add_lamports(snapshot_entry_lamports)?;

    // If this frame has no more entries, then close the frame account.
    if (snapshot_entry.id + 1) == snapshot_frame.total_entries {
        let snapshot_frame_lamports = snapshot_frame.get_lamports();
        snapshot_frame.sub_lamports(snapshot_frame_lamports)?;
        thread.add_lamports(snapshot_frame_lamports)?;

        // If this is also the last frame in the snapshot, then close the snapshot account.
        if (snapshot_frame.id + 1) == snapshot.total_frames {
            let snapshot_lamports = snapshot.get_lamports();
            snapshot.sub_lamports(snapshot_lamports)?;
            thread.add_lamports(snapshot_frame_lamports)?;
        }
    }

    // Build the next instruction
    let dynamic_instruction = if (snapshot_entry.id + 1) < snapshot_frame.total_entries {
        // Move on to the next entry.
        Some (
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DeleteSnapshotProcessEntry {
                    config:config.key(),
                    registry:registry.key(),
                    snapshot:snapshot.key(),
                    snapshot_entry:SnapshotEntry::pubkey(snapshot_frame.key(), snapshot_entry.id + 1),
                    snapshot_frame:snapshot_frame.key(),
                    thread: thread.key(),
                }.to_account_metas(Some(true)),
                data: crate::instruction::DeleteSnapshotProcessEntry{}.data()
            }.into()
        )
    } else if (snapshot_frame.id + 1) < snapshot.total_frames {
        // This frame has no more entries. Move onto the next frame.
        Some(Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::DeleteSnapshotProcessFrame {
                config: config.key(), 
                registry: registry.key(), 
                snapshot: snapshot.key(), 
                snapshot_frame: SnapshotFrame::pubkey(snapshot.key(), snapshot_frame.id + 1), 
                thread: thread.key(),
            }.to_account_metas(Some(true)),
            data: crate::instruction::DeleteSnapshotProcessFrame{}.data()
        }.into())
    } else {
        // This frame as no more entires and it was the last frame in the snapshot. We are done!
        None
    };

    Ok( ThreadResponse { dynamic_instruction, close_to: None, trigger: None } )
}
