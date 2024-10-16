use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn build_ix(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: sablier_network_program::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

pub fn get_anchor_account<T: AccountDeserialize>(svm: &mut LiteSVM, key: &Pubkey) -> T {
    let account = svm.get_account(key).unwrap();
    T::try_deserialize(&mut &account.data[..]).unwrap()
}
