use {anchor_lang::prelude::*, sablier_utils::CrateInfo};

/// Accounts required for the `get_crate_info` instruction.
/// We are not using system program actually
/// But anchor does not support empty structs: https://github.com/coral-xyz/anchor/pull/1659
#[derive(Accounts)]
pub struct GetCrateInfo<'info> {
    pub system_program: Program<'info, System>,
}

pub fn handler(_ctx: Context<GetCrateInfo>) -> Result<CrateInfo> {
    let spec = format!(
        "https://github.com/sablier-xyz/sablier/blob/v{}/programs/thread/Cargo.toml",
        version!()
    );
    let blob = "";
    let info = CrateInfo {
        spec,
        blob: blob.into(),
    };
    msg!("{}", info);

    Ok(info)
}
