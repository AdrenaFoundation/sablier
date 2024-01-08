use crate::constants::{SEED_WEBHOOK, TIMEOUT_THRESHOLD};

use {
    crate::state::Webhook,
    anchor_lang::{prelude::*, system_program},
};

#[derive(Accounts)]
#[instruction()]
pub struct WebhookRespond<'info> {
    #[account(mut)]
    pub ack_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_WEBHOOK,
            webhook.authority.as_ref(),
        ],
        bump,
        // close = caller,
        // has_one = caller
    )]
    pub webhook: Account<'info, Webhook>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account()]
    pub worker: SystemAccount<'info>,
}

pub fn handler(ctx: Context<WebhookRespond>) -> Result<()> {
    // Get accounts
    // let fee = &mut ctx.accounts.fee;
    let webhook = &mut ctx.accounts.webhook;
    let worker = &mut ctx.accounts.worker;

    // Payout webhook fee
    let current_slot = Clock::get().unwrap().slot;
    let _is_authorized_worker = webhook.workers.contains(&worker.key());
    let _is_within_execution_window = current_slot < webhook.created_at + TIMEOUT_THRESHOLD;
    // if is_authorized_worker && is_within_execution_window {
    //     // Pay worker for executing webhook
    //     // fee.pay_to_worker(webhook)?;
    // } else {
    //     // Either someone is spamming or this webhook has timed out. Do not pay worker.
    //     // TODO Perhaps rather than being paid to the admin, this could be put in an escrow account where all workers could claim equal rewards.
    //     // TODO If not claimed within X slots, the admin can claim their rewards and close the account.
    //     // fee.pay_to_admin(webhook)?;
    // }

    Ok(())
}
