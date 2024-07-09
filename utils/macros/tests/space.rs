use sablier_utils::MinSpace;
use sablier_utils::Space;
use solana_sdk::pubkey::Pubkey;

mod inside_mod {
    use sablier_utils::MinSpace;

    #[derive(MinSpace)]
    pub struct Data {
        pub data: u64,
    }
}

#[derive(MinSpace)]
pub enum TestBasicEnum {
    Basic1,
    Basic2 {
        test_u8: u8,
    },
    Basic3 {
        test_u16: u16,
    },
    Basic4 {
        #[max_len(10)]
        test_vec: Vec<u8>,
    },
}

#[derive(MinSpace)]
pub struct TestEmptyAccount {}

#[derive(MinSpace)]
pub struct TestBasicVarAccount {
    pub test_u8: u8,
    pub test_u16: u16,
    pub test_u32: u32,
    pub test_u64: u64,
    pub test_u128: u128,
}

#[derive(MinSpace)]
pub struct TestComplexeVarAccount {
    pub test_key: Pubkey,
    #[max_len(10)]
    pub test_vec: Vec<u8>,
    #[max_len(10)]
    pub test_string: String,
    pub test_option: Option<u16>,
}

#[derive(MinSpace)]
pub struct TestNonAccountStruct {
    pub test_bool: bool,
}

#[derive(MinSpace)]
pub struct TestZeroCopyStruct {
    pub test_array: [u8; 8],
    pub test_u32: u32,
}

#[derive(MinSpace)]
pub struct ChildStruct {
    #[max_len(10)]
    pub test_string: String,
}

#[derive(MinSpace)]
pub struct TestNestedStruct {
    pub test_struct: ChildStruct,
    pub test_enum: TestBasicEnum,
}

#[derive(MinSpace)]
pub struct TestMatrixStruct {
    #[max_len(2, 4)]
    pub test_matrix: Vec<Vec<u8>>,
}

#[derive(MinSpace)]
pub struct TestFullPath {
    pub test_option_path: Option<inside_mod::Data>,
    pub test_path: inside_mod::Data,
}

const MAX_LEN: u8 = 10;

#[derive(MinSpace)]
pub struct TestConst {
    #[max_len(MAX_LEN)]
    pub test_string: String,
    pub test_array: [u8; MAX_LEN as usize],
}

#[derive(MinSpace)]
pub struct TestRaw {
    #[raw_space(100)]
    pub test_string: String,
    #[max_len(2, raw_space(10))]
    pub test_matrix: Vec<Vec<u8>>,
}

#[test]
fn test_empty_struct() {
    assert_eq!(TestEmptyAccount::MIN_SPACE, 0);
}

#[test]
fn test_basic_struct() {
    assert_eq!(TestBasicVarAccount::MIN_SPACE, 1 + 2 + 4 + 8 + 16);
}

#[test]
fn test_complexe_struct() {
    assert_eq!(
        TestComplexeVarAccount::MIN_SPACE,
        32 + 4 + 10 + (4 + 10) + 3
    )
}

#[test]
fn test_zero_copy_struct() {
    assert_eq!(TestZeroCopyStruct::MIN_SPACE, 8 + 4)
}

#[test]
fn test_basic_enum() {
    assert_eq!(TestBasicEnum::MIN_SPACE, 1 + 14);
}

#[test]
fn test_nested_struct() {
    assert_eq!(
        TestNestedStruct::MIN_SPACE,
        ChildStruct::MIN_SPACE + TestBasicEnum::MIN_SPACE
    )
}

#[test]
fn test_matrix_struct() {
    assert_eq!(TestMatrixStruct::MIN_SPACE, 4 + (2 * (4 + 4)))
}

#[test]
fn test_full_path() {
    assert_eq!(TestFullPath::MIN_SPACE, 8 + 9)
}

#[test]
fn test_const() {
    assert_eq!(TestConst::MIN_SPACE, (4 + 10) + 10)
}

#[test]
fn test_raw() {
    assert_eq!(TestRaw::MIN_SPACE, 100 + 4 + 2 * 10);
}
