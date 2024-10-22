use std::collections::VecDeque;

use anchor_lang::system_program;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use sablier_network_program::state::{Config, Pool, Registry};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

use crate::utils::{build_ix, get_anchor_account};

pub fn pool_create(svm: &mut LiteSVM, admin_kp: &Keypair) -> Result<(), FailedTransactionMetadata> {
    let admin = admin_kp.pubkey();

    let ix = build_ix(
        sablier_network_program::accounts::PoolCreate {
            config: Config::pubkey(),
            system_program: system_program::ID,
            payer: admin,
            pool: Pool::pubkey(0),
            admin,
            registry: Registry::pubkey(),
        },
        sablier_network_program::instruction::PoolCreate {},
    );

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin), &[&admin_kp], blockhash);
    svm.send_transaction(tx)?;

    // CHECKS

    let registry: Registry = get_anchor_account(svm, &Registry::pubkey());
    assert_eq!(registry.current_epoch, 0);
    assert_eq!(registry.locked, false);
    assert_eq!(registry.nonce, 0);
    assert_eq!(registry.total_pools, 1);
    assert_eq!(registry.total_unstakes, 0);
    assert_eq!(registry.total_workers, 0);

    let pool: Pool = get_anchor_account(svm, &Pool::pubkey(0));
    assert_eq!(pool.id, 0);
    assert_eq!(pool.size, 1);
    assert_eq!(pool.workers, VecDeque::new());

    Ok(())
}
