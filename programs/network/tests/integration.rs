use litesvm::LiteSVM;
use litesvm_token::CreateMint;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};

mod test_instructions;
mod utils;

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let path = format!(
        "{}/../deploy/sablier_network_program.so",
        env!("CARGO_TARGET_TMPDIR")
    );
    svm.add_program_from_file(sablier_network_program::ID, path)
        .unwrap();

    let admin_kp = Keypair::new();

    svm.airdrop(&admin_kp.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();

    (svm, admin_kp)
}

#[test]
pub fn integration_test() {
    let (mut svm, admin_kp) = setup();

    let mint = CreateMint::new(&mut svm, &admin_kp).send().unwrap();

    test_instructions::initialize(&mut svm, &admin_kp, &mint).unwrap();
    test_instructions::pool_create(&mut svm, &admin_kp).unwrap();
    test_instructions::worker_create(&mut svm, &admin_kp, &mint).unwrap();
}
