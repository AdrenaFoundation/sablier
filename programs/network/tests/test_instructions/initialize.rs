use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use sablier_network_program::{
    constants::SEED_REGISTRY,
    state::{Config, Registry, Snapshot},
};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction,
};

use crate::utils::{build_ix, get_anchor_account};

pub fn initialize(
    svm: &mut LiteSVM,
    admin_kp: &Keypair,
    mint: &Pubkey,
) -> Result<(), FailedTransactionMetadata> {
    let admin = admin_kp.pubkey();

    let (registry, registry_bump) =
        Pubkey::find_program_address(&[SEED_REGISTRY], &sablier_network_program::ID);
    let ix = build_ix(
        sablier_network_program::accounts::Initialize {
            admin,
            config: Config::pubkey(),
            mint: *mint,
            registry,
            snapshot: Snapshot::pubkey(0),
            system_program: system_program::ID,
        },
        sablier_network_program::instruction::Initialize {},
    );

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin), &[&admin_kp], blockhash);
    svm.send_transaction(tx)?;

    // CHECKS

    let config: Config = get_anchor_account(svm, &Config::pubkey());
    assert_eq!(config.admin, admin);
    assert_eq!(config.mint, *mint);
    assert_eq!(config.hasher_thread, Pubkey::default());
    assert_eq!(config.hasher_thread, Pubkey::default());

    let registry: Registry = get_anchor_account(svm, &Registry::pubkey());
    assert_eq!(registry.current_epoch, 0);
    assert_eq!(registry.locked, false);
    assert_eq!(registry.nonce, 0);
    assert_eq!(registry.total_pools, 0);
    assert_eq!(registry.total_unstakes, 0);
    assert_eq!(registry.total_workers, 0);
    assert_eq!(registry.bump, registry_bump);

    let snapshot: Snapshot = get_anchor_account(svm, &Snapshot::pubkey(0));
    assert_eq!(snapshot.id, 0);
    assert_eq!(snapshot.total_frames, 0);
    assert_eq!(snapshot.total_stake, 0);

    Ok(())
}
