use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use sablier_utils::thread::ThreadResponse;

use crate::{constants::*, state::*};

#[derive(Accounts)]
pub struct DistributeFeesProcessFrame<'info> {
    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        mut,
        seeds = [
            SEED_FEE,
            fee.worker.as_ref(),
        ],
        bump,
        has_one = worker,
    )]
    pub fee: Account<'info, Fee>,

    #[account(address = Registry::pubkey())]
    pub registry: Account<'info, Registry>,

    #[account(
        address = snapshot.pubkey(),
        constraint = snapshot.id == registry.current_epoch
    )]
    pub snapshot: Account<'info, Snapshot>,

    #[account(
        address = snapshot_frame.pubkey(),
        has_one = snapshot,
        has_one = worker,
    )]
    pub snapshot_frame: Account<'info, SnapshotFrame>,

    #[account(address = config.load()?.epoch_thread)]
    pub thread: Signer<'info>,

    #[account(mut)]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<DistributeFeesProcessFrame>) -> Result<ThreadResponse> {
    // Get accounts.
    let config = &ctx.accounts.config;
    let fee = &mut ctx.accounts.fee;
    let registry = &ctx.accounts.registry;
    let snapshot = &ctx.accounts.snapshot;
    let snapshot_frame = &ctx.accounts.snapshot_frame;
    let thread = &ctx.accounts.thread;
    let worker = &mut ctx.accounts.worker;

    // Calculate the fee account's usuable balance.
    let fee_lamport_balance = fee.get_lamports();
    let fee_data_len = 8 + Fee::INIT_SPACE;
    let fee_rent_balance = Rent::get()?.minimum_balance(fee_data_len);
    let fee_usable_balance = fee_lamport_balance - fee_rent_balance;

    // Calculate the commission to be retained by the worker.
    let commission_balance = fee_usable_balance * worker.commission_rate / 100;

    // Transfer commission to the worker.
    fee.sub_lamports(commission_balance)?;
    worker.add_lamports(commission_balance)?;

    // Increment the worker's commission balance.
    worker.commission_balance += commission_balance;

    // Record the balance that is distributable to delegations.
    fee.distributable_balance = fee_usable_balance - commission_balance;

    // Build next instruction for the thread.
    let dynamic_instruction = if snapshot_frame.total_entries > 0 {
        // This snapshot frame has entries. Distribute fees to the delegations associated with the entries.
        let delegation_pubkey = Delegation::pubkey(worker.key(), 0);
        let snapshot_entry_pubkey = SnapshotEntry::pubkey(snapshot_frame.key(), 0);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DistributeFeesProcessEntry {
                    config: config.key(),
                    delegation: delegation_pubkey,
                    fee: fee.key(),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_entry: snapshot_entry_pubkey.key(),
                    snapshot_frame: snapshot_frame.key(),
                    thread: thread.key(),
                    worker: worker.key(),
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::DistributeFeesProcessEntry {}.data(),
            }
            .into(),
        )
    } else if (snapshot_frame.id + 1) < snapshot.total_frames {
        // This frame has no entries. Move on to the next frame.
        let next_worker_pubkey = Worker::pubkey(worker.id + 1);
        let next_snapshot_frame_pubkey =
            SnapshotFrame::pubkey(snapshot.key(), snapshot_frame.id + 1);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DistributeFeesProcessFrame {
                    config: config.key(),
                    fee: Fee::pubkey(next_worker_pubkey),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_frame: next_snapshot_frame_pubkey,
                    thread: thread.key(),
                    worker: next_worker_pubkey,
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::DistributeFeesProcessFrame {}.data(),
            }
            .into(),
        )
    } else {
        None
    };

    Ok(ThreadResponse {
        dynamic_instruction,
        close_to: None,
        trigger: None,
    })
}
