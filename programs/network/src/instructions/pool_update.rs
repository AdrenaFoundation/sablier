use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        system_program::{transfer, Transfer},
    },
    sablier_utils::account::AccountInfoExt,
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(settings: PoolSettings)]
pub struct PoolUpdate<'info> {
    pub admin: Signer<'info>,

    #[account(
        address = Config::pubkey(),
        has_one = admin
    )]
    pub config: AccountLoader<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, address = pool.pubkey())]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PoolUpdate>, settings: PoolSettings) -> Result<()> {
    // Get accounts
    let payer = &ctx.accounts.payer;
    let pool = &mut ctx.accounts.pool;
    let system_program = &ctx.accounts.system_program;

    // Update the pool settings
    pool.update(&settings)?;

    // Reallocate memory for the pool account
    let data_len = 8 + size_of::<Pool>() + settings.size.checked_mul(size_of::<Pubkey>()).unwrap();
    pool.realloc(data_len, false)?;

    // If lamports are required to maintain rent-exemption, pay them
    let minimum_rent = Rent::get()?.minimum_balance(data_len);
    if minimum_rent > pool.get_lamports() {
        transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer.to_account_info(),
                    to: pool.to_account_info(),
                },
            ),
            minimum_rent - pool.get_lamports(),
        )?;
    }

    Ok(())
}
