use miden_core::FieldElement;

use super::{Felt, StorageItem};

pub const ACCOUNT_SEED_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN: [u64; 4] = [
    5950491586293629690,
    3173174058297886549,
    16553747801483039178,
    11841717777847436894,
];
pub const ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN: u64 = 3972335011818762557;
pub const ACCOUNT_ID_SENDER: u64 = 0b0110111011u64 << 54;

pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN: u64 = 0b1010011100 << 54;
pub const ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN: u64 = 0b1110011100 << 54;

pub const NON_FUNGIBLE_ASSET_DATA: [u8; 4] = [1, 2, 3, 4];

pub const NONCE: Felt = Felt::ZERO;

pub const STORAGE_INDEX_0: u8 = 20;
pub const STORAGE_VALUE_0: [Felt; 4] = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
pub const STORAGE_INDEX_1: u8 = 30;
pub const STORAGE_VALUE_1: [Felt; 4] = [Felt::new(5), Felt::new(6), Felt::new(7), Felt::new(8)];
pub const STORAGE_ITEM_0: StorageItem = (STORAGE_INDEX_0, STORAGE_VALUE_0);
pub const STORAGE_ITEM_1: StorageItem = (STORAGE_INDEX_1, STORAGE_VALUE_1);

pub const CHILD_ROOT_PARENT_LEAF_INDEX: u8 = 10;
pub const CHILD_SMT_DEPTH: u8 = 64;
pub const CHILD_STORAGE_INDEX_0: u64 = 40;
pub const CHILD_STORAGE_VALUE_0: [Felt; 4] =
    [Felt::new(11), Felt::new(12), Felt::new(13), Felt::new(14)];
