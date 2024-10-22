use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use sablier_network_program::state::{Config, Registry, Worker};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::utils::{build_ix, get_anchor_account};

pub fn worker_create(
    svm: &mut LiteSVM,
    admin_kp: &Keypair,
    mint: &Pubkey,
) -> Result<(), FailedTransactionMetadata> {
    let admin = admin_kp.pubkey();
    let worker_authority = Keypair::new();

    svm.airdrop(&worker_authority.pubkey(), 2 * LAMPORTS_PER_SOL)?;

    let worker = Worker::pubkey(0);
    let worker_tokens = get_associated_token_address(&worker, mint);

    let ix = build_ix(
        sablier_network_program::accounts::WorkerCreate {
            authority: admin,
            config: Config::pubkey(),
            mint: *mint,
            registry: Registry::pubkey(),
            signatory: worker_authority.pubkey(),
            worker,
            worker_tokens,
            system_program: system_program::ID,
            associated_token_program: spl_associated_token_account::ID,
            token_program: spl_token::ID,
        },
        sablier_network_program::instruction::WorkerCreate {},
    );

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin),
        &[&admin_kp, &worker_authority],
        blockhash,
    );
    svm.send_transaction(tx)?;

    let worker: Worker = get_anchor_account(svm, &Worker::pubkey(0));
    assert_eq!(worker.id, 0);
    assert_eq!(worker.total_delegations, 0);
    assert_eq!(worker.authority, admin);
    assert_eq!(worker.commission_balance, 0);
    assert_eq!(worker.commission_rate, 0);
    assert_eq!(worker.signatory, worker_authority.pubkey());

    let registry: Registry = get_anchor_account(svm, &Registry::pubkey());
    assert_eq!(registry.current_epoch, 0);
    assert_eq!(registry.locked, false);
    assert_eq!(registry.nonce, 0);
    assert_eq!(registry.total_pools, 1);
    assert_eq!(registry.total_unstakes, 0);
    assert_eq!(registry.total_workers, 1);

    Ok(())
}
