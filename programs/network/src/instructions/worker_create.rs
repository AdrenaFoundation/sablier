use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct WorkerCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        init,
        seeds = [
            SEED_FEE,
            worker.key().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Fee::INIT_SPACE,
    )]
    pub fee: AccountLoader<'info, Fee>,

    #[account(address = config.load()?.mint)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
        constraint = !registry.locked @ SablierError::RegistryLocked
    )]
    pub registry: Box<Account<'info, Registry>>,

    #[account(
        init,
        seeds = [
            SEED_WORKER,
            registry.total_workers.to_be_bytes().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Worker::INIT_SPACE,
    )]
    pub worker: Box<Account<'info, Worker>>,

    #[account(
        init,
        payer = authority,
        associated_token::authority = worker,
        associated_token::mint = mint,
    )]
    pub worker_tokens: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, WorkerCreate<'info>>) -> Result<()> {
    // Get accounts
    let authority = &mut ctx.accounts.authority;
    let fee = &mut ctx.accounts.fee;
    let registry = &mut ctx.accounts.registry;
    let worker = &mut ctx.accounts.worker;

    let signatory = {
        let signatory_info = ctx
            .remaining_accounts
            .first()
            .ok_or(ErrorCode::AccountNotEnoughKeys)?;

        if !signatory_info.is_signer {
            return Err(ErrorCode::AccountNotSigner.into());
        }

        if signatory_info.key == authority.key {
            return Err(SablierError::InvalidSignatory.into());
        }

        Signer::try_from(signatory_info)?
    };

    let mut penalty: Account<Penalty> = {
        let penalty_info = ctx
            .remaining_accounts
            .get(1)
            .ok_or(ErrorCode::AccountNotEnoughKeys)?;

        if !penalty_info.is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        let (pda_key, bump) =
            Pubkey::find_program_address(&[SEED_PENALTY, worker.key().as_ref()], &crate::ID);

        if &pda_key != penalty_info.key {
            return Err(ErrorCode::ConstraintSeeds.into());
        }

        let account_space = 8 + Penalty::INIT_SPACE;
        let lamports = Rent::get()?.minimum_balance(account_space);
        let cpi_accounts = anchor_lang::system_program::CreateAccount {
            from: authority.to_account_info(),
            to: penalty_info.to_owned(),
        };
        let cpi_context = anchor_lang::context::CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            cpi_accounts,
        );
        anchor_lang::system_program::create_account(
            cpi_context.with_signer(&[&[SEED_PENALTY, worker.key().as_ref(), &[bump][..]][..]]),
            lamports,
            account_space as u64,
            &crate::ID,
        )?;
        Account::try_from_unchecked(penalty_info)?
    };

    // Initialize the worker accounts.
    worker.init(authority, registry.total_workers, &signatory)?;
    fee.init(worker.key())?;
    penalty.init(worker.key())?;

    // Update the registry's worker counter.
    registry.total_workers += 1;

    Ok(())
}
