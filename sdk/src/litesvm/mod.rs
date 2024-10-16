use std::path::Path;

use litesvm::{
    types::{FailedTransactionMetadata, TransactionResult},
    LiteSVM,
};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

fn setup_test_environment(
    network_program_path: impl AsRef<Path>,
    thread_program_path: impl AsRef<Path>,
) -> Result<(), FailedTransactionMetadata> {
    let mut svm = LiteSVM::new();

    let sablier_admin_kp = Keypair::new();
    let sablier_admin_pk = sablier_admin_kp.pubkey();

    svm.airdrop(&sablier_admin_pk, 10 * LAMPORTS_PER_SOL)?;

    svm.add_program_from_file(sablier_network_program::id(), network_program_path.as_ref())
        .unwrap();
    svm.add_program_from_file(sablier_thread_program::id(), thread_program_path.as_ref())
        .unwrap();

    let mint = setup_sablier_token(&mut svm, &sablier_admin_kp)?;

    Ok(())
}

fn setup_sablier_token(
    svm: &mut LiteSVM,
    sablier_admin_kp: &Keypair,
) -> Result<Pubkey, FailedTransactionMetadata> {
    let mint = CreateMint::new(svm, sablier_admin_kp).send()?;

    let ata = CreateAssociatedTokenAccount::new(svm, sablier_admin_kp, &mint).send()?;
    MintTo::new(svm, sablier_admin_kp, &mint, &ata, 10).send()?;

    Ok(mint)
}
