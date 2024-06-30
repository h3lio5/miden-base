use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::fmt;

use super::{
    get_account_seed, AccountError, ByteReader, Deserializable, DeserializationError, Digest, Felt,
    Hasher, Serializable, Word, ZERO,
};
use crate::{crypto::merkle::LeafIndex, utils::hex_to_bytes, ACCOUNT_TREE_DEPTH};

// CONSTANTS
// ================================================================================================

// The higher two bits of the most significant nibble determines the account storage mode
pub const ACCOUNT_STORAGE_MASK_SHIFT: u64 = 62;
pub const ACCOUNT_STORAGE_MASK: u64 = 0b11 << ACCOUNT_STORAGE_MASK_SHIFT;

// The lower two bits of the most significant nibble determines the account type
pub const ACCOUNT_TYPE_MASK_SHIFT: u64 = 60;
pub const ACCOUNT_TYPE_MASK: u64 = 0b11 << ACCOUNT_TYPE_MASK_SHIFT;
pub const ACCOUNT_ISFAUCET_MASK: u64 = 0b10 << ACCOUNT_TYPE_MASK_SHIFT;

// ACCOUNT TYPES
// ================================================================================================

pub const FUNGIBLE_FAUCET: u64 = 0b10;
pub const NON_FUNGIBLE_FAUCET: u64 = 0b11;
pub const REGULAR_ACCOUNT_IMMUTABLE_CODE: u64 = 0b00;
pub const REGULAR_ACCOUNT_UPDATABLE_CODE: u64 = 0b01;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum AccountType {
    FungibleFaucet = FUNGIBLE_FAUCET,
    NonFungibleFaucet = NON_FUNGIBLE_FAUCET,
    RegularAccountImmutableCode = REGULAR_ACCOUNT_IMMUTABLE_CODE,
    RegularAccountUpdatableCode = REGULAR_ACCOUNT_UPDATABLE_CODE,
}

/// Returns the [AccountType] given an integer representation of `account_id`.
impl From<u64> for AccountType {
    fn from(value: u64) -> Self {
        debug_assert!(
            ACCOUNT_TYPE_MASK.count_ones() == 2,
            "This method assumes there are only 2bits in the mask"
        );

        let bits = (value & ACCOUNT_TYPE_MASK) >> ACCOUNT_TYPE_MASK_SHIFT;
        match bits {
            REGULAR_ACCOUNT_UPDATABLE_CODE => AccountType::RegularAccountUpdatableCode,
            REGULAR_ACCOUNT_IMMUTABLE_CODE => AccountType::RegularAccountImmutableCode,
            FUNGIBLE_FAUCET => AccountType::FungibleFaucet,
            NON_FUNGIBLE_FAUCET => AccountType::NonFungibleFaucet,
            _ => {
                unreachable!("account_type mask contains only 2bits, there are 4 options total")
            },
        }
    }
}

// ACCOUNT STORAGE TYPES
// ================================================================================================

pub const ON_CHAIN: u64 = 0b00;
pub const OFF_CHAIN: u64 = 0b10;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum AccountStorageType {
    OnChain = ON_CHAIN,
    OffChain = OFF_CHAIN,
}

// ACCOUNT ID
// ================================================================================================

/// Unique identifier of an account.
///
/// Account ID consists of 1 field element (~64 bits). The most significant bits in the id are used
/// to encode the account' storage and type.
///
/// The top two bits are used to encode the storage type. The values [OFF_CHAIN] and [ON_CHAIN]
/// encode the account's storage type. The next two bits encode the account type. The values
/// [FUNGIBLE_FAUCET], [NON_FUNGIBLE_FAUCET], [REGULAR_ACCOUNT_IMMUTABLE_CODE], and
/// [REGULAR_ACCOUNT_UPDATABLE_CODE] encode the account's type.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct AccountId(Felt);

impl AccountId {
    /// Specifies a minimum number of trailing zeros required in the last element of the seed digest.
    ///
    /// Note: The account id includes 4 bits of metadata, these bits determine the account type
    /// (normal account, fungible token, non-fungible token), the storage type (on/off chain), and
    /// for the normal accounts if the code is updatable or not. These metadata bits are also
    /// checked by the PoW and add to the total work defined below.
    #[cfg(not(any(feature = "testing", test)))]
    pub const REGULAR_ACCOUNT_SEED_DIGEST_MIN_TRAILING_ZEROS: u32 = 23;
    #[cfg(not(any(feature = "testing", test)))]
    pub const FAUCET_SEED_DIGEST_MIN_TRAILING_ZEROS: u32 = 31;
    #[cfg(any(feature = "testing", test))]
    pub const REGULAR_ACCOUNT_SEED_DIGEST_MIN_TRAILING_ZEROS: u32 = 5;
    #[cfg(any(feature = "testing", test))]
    pub const FAUCET_SEED_DIGEST_MIN_TRAILING_ZEROS: u32 = 6;

    /// Specifies a minimum number of ones for a valid account ID.
    pub const MIN_ACCOUNT_ONES: u32 = 5;

    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new account ID derived from the specified seed, code root and storage root.
    ///
    /// The account ID is computed by hashing the seed, code root and storage root and using 1
    /// element of the resulting digest to form the ID. Specifically we take element 0. We also
    /// require that the last element of the seed digest has at least `23` trailing zeros if it
    /// is a regular account, or `31` trailing zeros if it is a faucet account.
    ///
    /// The seed digest is computed using a sequential hash over
    /// hash(SEED, CODE_ROOT, STORAGE_ROOT, ZERO).  This takes two permutations.
    ///
    /// # Errors
    /// Returns an error if the resulting account ID does not comply with account ID rules:
    /// - the metadata embedded in the ID (i.e., the first 4 bits) is valid.
    /// - the ID has at least `5` ones.
    /// - the last element of the seed digest has at least `23` trailing zeros for regular
    ///   accounts.
    /// - the last element of the seed digest has at least `31` trailing zeros for faucet accounts.
    pub fn new(seed: Word, code_root: Digest, storage_root: Digest) -> Result<Self, AccountError> {
        let seed_digest = compute_digest(seed, code_root, storage_root);

        Self::validate_seed_digest(&seed_digest)?;
        seed_digest[0].try_into()
    }

    /// Creates a new [AccountId] without checking its validity.
    ///
    /// This function requires that the provided value is a valid [Felt] representation of an
    /// [AccountId].
    pub fn new_unchecked(value: Felt) -> Self {
        Self(value)
    }

    /// Creates a new dummy [AccountId] for testing purposes.
    #[cfg(any(feature = "testing", test))]
    pub fn new_dummy(init_seed: [u8; 32], account_type: AccountType) -> Self {
        let code_root = Digest::default();
        let storage_root = Digest::default();

        let seed = get_account_seed(
            init_seed,
            account_type,
            AccountStorageType::OnChain,
            code_root,
            storage_root,
        )
        .unwrap();

        Self::new(seed, code_root, storage_root).unwrap()
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the type of this account ID.
    pub fn account_type(&self) -> AccountType {
        self.0.as_int().into()
    }

    /// Returns true if an account with this ID is a faucet (can issue assets).
    pub fn is_faucet(&self) -> bool {
        matches!(
            self.account_type(),
            AccountType::FungibleFaucet | AccountType::NonFungibleFaucet
        )
    }

    /// Returns true if an account with this ID is a regular account.
    pub fn is_regular_account(&self) -> bool {
        is_regular_account(self.0.as_int())
    }

    /// Returns the storage type of this account (e.g., on-chain or off-chain).
    pub fn storage_type(&self) -> AccountStorageType {
        let bits = (self.0.as_int() & ACCOUNT_STORAGE_MASK) >> ACCOUNT_STORAGE_MASK_SHIFT;
        match bits {
            ON_CHAIN => AccountStorageType::OnChain,
            OFF_CHAIN => AccountStorageType::OffChain,
            _ => panic!("Account with invalid storage bits created"),
        }
    }

    /// Returns true if an account with this ID is an on-chain account.
    pub fn is_on_chain(&self) -> bool {
        self.storage_type() == AccountStorageType::OnChain
    }

    /// Finds and returns a seed suitable for creating an account ID for the specified account type
    /// using the provided initial seed as a starting point.
    pub fn get_account_seed(
        init_seed: [u8; 32],
        account_type: AccountType,
        storage_type: AccountStorageType,
        code_root: Digest,
        storage_root: Digest,
    ) -> Result<Word, AccountError> {
        get_account_seed(init_seed, account_type, storage_type, code_root, storage_root)
    }

    /// Creates an Account Id from a hex string. Assumes the string starts with "0x" and
    /// that the hexadecimal characters are big-endian encoded.
    pub fn from_hex(hex_value: &str) -> Result<AccountId, AccountError> {
        hex_to_bytes(hex_value)
            .map_err(|err| AccountError::HexParseError(err.to_string()))
            .and_then(|mut bytes: [u8; 8]| {
                // `bytes` ends up being parsed as felt, and the input to that is assumed to be little-endian
                // so we need to reverse the order
                bytes.reverse();
                bytes.try_into()
            })
    }

    /// Returns a big-endian, hex-encoded string.
    pub fn to_hex(&self) -> String {
        format!("0x{:016x}", self.0.as_int())
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns an error if:
    /// - There are fewer then:
    ///   - 24 trailing ZEROs in the last element of the seed digest for regular accounts.
    ///   - 32 trailing ZEROs in the last element of the seed digest for faucet accounts.
    pub(super) fn validate_seed_digest(digest: &Digest) -> Result<(), AccountError> {
        // check the id satisfies the proof-of-work requirement.
        let required_zeros = if is_regular_account(digest[0].as_int()) {
            Self::REGULAR_ACCOUNT_SEED_DIGEST_MIN_TRAILING_ZEROS
        } else {
            Self::FAUCET_SEED_DIGEST_MIN_TRAILING_ZEROS
        };

        let trailing_zeros = digest_pow(*digest);
        if required_zeros > trailing_zeros {
            return Err(AccountError::seed_digest_too_few_trailing_zeros(
                required_zeros,
                trailing_zeros,
            ));
        }

        Ok(())
    }
}

impl PartialOrd for AccountId {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccountId {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.as_int().cmp(&other.0.as_int())
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0.as_int())
    }
}

// CONVERSIONS FROM ACCOUNT ID
// ================================================================================================

impl From<AccountId> for Felt {
    fn from(id: AccountId) -> Self {
        id.0
    }
}

impl From<AccountId> for [u8; 8] {
    fn from(id: AccountId) -> Self {
        let mut result = [0_u8; 8];
        result[..8].copy_from_slice(&id.0.as_int().to_le_bytes());
        result
    }
}

impl From<AccountId> for u64 {
    fn from(id: AccountId) -> Self {
        id.0.as_int()
    }
}

/// Account IDs are used as indexes in the account database, which is a tree of depth 64.
impl From<AccountId> for LeafIndex<ACCOUNT_TREE_DEPTH> {
    fn from(id: AccountId) -> Self {
        LeafIndex::new_max_depth(id.0.as_int())
    }
}

// CONVERSIONS TO ACCOUNT ID
// ================================================================================================

impl TryFrom<Felt> for AccountId {
    type Error = AccountError;

    /// Returns an [AccountId] instantiated with the provided field element.
    ///
    /// # Errors
    /// Returns an error if:
    /// - If there are fewer than [AccountId::MIN_ACCOUNT_ONES] in the provided value.
    /// - If the provided value contains invalid account ID metadata (i.e., the first 4 bits).
    fn try_from(value: Felt) -> Result<Self, Self::Error> {
        let int_value = value.as_int();

        let count = int_value.count_ones();
        if count < Self::MIN_ACCOUNT_ONES {
            return Err(AccountError::account_id_too_few_ones(Self::MIN_ACCOUNT_ONES, count));
        }

        let bits = (int_value & ACCOUNT_STORAGE_MASK) >> ACCOUNT_STORAGE_MASK_SHIFT;
        match bits {
            ON_CHAIN | OFF_CHAIN => (),
            _ => return Err(AccountError::InvalidAccountStorageType),
        };

        Ok(Self(value))
    }
}

impl TryFrom<[u8; 8]> for AccountId {
    type Error = AccountError;

    // Expects little-endian byte order
    fn try_from(value: [u8; 8]) -> Result<Self, Self::Error> {
        let element = parse_felt(&value[..8])?;
        Self::try_from(element)
    }
}

impl TryFrom<u64> for AccountId {
    type Error = AccountError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let element = parse_felt(&value.to_le_bytes())?;
        Self::try_from(element)
    }
}

// SERIALIZATION
// ================================================================================================

impl Serializable for AccountId {
    fn write_into<W: miden_crypto::utils::ByteWriter>(&self, target: &mut W) {
        self.0.write_into(target);
    }
}

impl Deserializable for AccountId {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        Felt::read_from(source)?
            .try_into()
            .map_err(|err: AccountError| DeserializationError::InvalidValue(err.to_string()))
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn parse_felt(bytes: &[u8]) -> Result<Felt, AccountError> {
    Felt::try_from(bytes).map_err(|err| AccountError::AccountIdInvalidFieldElement(err.to_string()))
}

/// Returns the digest of two hashing permutations over the seed, code root, storage root and
/// padding.
pub(super) fn compute_digest(seed: Word, code_root: Digest, storage_root: Digest) -> Digest {
    let mut elements = Vec::with_capacity(16);
    elements.extend(seed);
    elements.extend(*code_root);
    elements.extend(*storage_root);
    elements.resize(16, ZERO);
    Hasher::hash_elements(&elements)
}

/// Given a [Digest] returns its proof-of-work.
pub(super) fn digest_pow(digest: Digest) -> u32 {
    digest.as_elements()[3].as_int().trailing_zeros()
}

/// Returns true if an account with this ID is a regular account.
fn is_regular_account(account_id: u64) -> bool {
    let account_type = account_id.into();
    matches!(
        account_type,
        AccountType::RegularAccountUpdatableCode | AccountType::RegularAccountImmutableCode
    )
}

// TESTING
// ================================================================================================

#[cfg(any(feature = "testing", test))]
pub mod testing {
    use super::{
        AccountStorageType, AccountType, ACCOUNT_STORAGE_MASK_SHIFT, ACCOUNT_TYPE_MASK_SHIFT,
    };

    // CONSTANTS
    // --------------------------------------------------------------------------------------------

    // REGULAR ACCOUNTS - OFF-CHAIN
    pub const ACCOUNT_ID_SENDER: u64 = account_id(
        AccountType::RegularAccountImmutableCode,
        AccountStorageType::OffChain,
        0b0001_1111,
    );
    pub const ACCOUNT_ID_OFF_CHAIN_SENDER: u64 = account_id(
        AccountType::RegularAccountImmutableCode,
        AccountStorageType::OffChain,
        0b0010_1111,
    );
    pub const ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN: u64 = account_id(
        AccountType::RegularAccountUpdatableCode,
        AccountStorageType::OffChain,
        0b0011_1111,
    );
    // REGULAR ACCOUNTS - ON-CHAIN
    pub const ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN: u64 = account_id(
        AccountType::RegularAccountImmutableCode,
        AccountStorageType::OnChain,
        0b0001_1111,
    );
    pub const ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN_2: u64 = account_id(
        AccountType::RegularAccountImmutableCode,
        AccountStorageType::OnChain,
        0b0010_1111,
    );
    pub const ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN: u64 = account_id(
        AccountType::RegularAccountUpdatableCode,
        AccountStorageType::OnChain,
        0b0011_1111,
    );
    pub const ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN_2: u64 = account_id(
        AccountType::RegularAccountUpdatableCode,
        AccountStorageType::OnChain,
        0b0100_1111,
    );

    // FUNGIBLE TOKENS - OFF-CHAIN
    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_OFF_CHAIN: u64 =
        account_id(AccountType::FungibleFaucet, AccountStorageType::OffChain, 0b0001_1111);
    // FUNGIBLE TOKENS - ON-CHAIN
    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN: u64 =
        account_id(AccountType::FungibleFaucet, AccountStorageType::OnChain, 0b0001_1111);
    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN_1: u64 =
        account_id(AccountType::FungibleFaucet, AccountStorageType::OnChain, 0b0010_1111);
    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN_2: u64 =
        account_id(AccountType::FungibleFaucet, AccountStorageType::OnChain, 0b0011_1111);
    pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN_3: u64 =
        account_id(AccountType::FungibleFaucet, AccountStorageType::OnChain, 0b0100_1111);

    // NON-FUNGIBLE TOKENS - OFF-CHAIN
    pub const ACCOUNT_ID_INSUFFICIENT_ONES: u64 =
        account_id(AccountType::NonFungibleFaucet, AccountStorageType::OffChain, 0b0000_0000); // invalid
    pub const ACCOUNT_ID_NON_FUNGIBLE_FAUCET_OFF_CHAIN: u64 =
        account_id(AccountType::NonFungibleFaucet, AccountStorageType::OffChain, 0b0001_1111);
    // NON-FUNGIBLE TOKENS - ON-CHAIN
    pub const ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN: u64 =
        account_id(AccountType::NonFungibleFaucet, AccountStorageType::OnChain, 0b0010_1111);
    pub const ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN_1: u64 =
        account_id(AccountType::NonFungibleFaucet, AccountStorageType::OnChain, 0b0011_1111);

    // UTILITIES
    // --------------------------------------------------------------------------------------------

    pub const fn account_id(
        account_type: AccountType,
        storage: AccountStorageType,
        rest: u64,
    ) -> u64 {
        let mut id = 0;

        id ^= (storage as u64) << ACCOUNT_STORAGE_MASK_SHIFT;
        id ^= (account_type as u64) << ACCOUNT_TYPE_MASK_SHIFT;
        id ^= rest;

        id
    }
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    use miden_crypto::utils::{Deserializable, Serializable};

    use super::{
        testing::*, AccountId, AccountStorageType, AccountType, ACCOUNT_ISFAUCET_MASK,
        ACCOUNT_TYPE_MASK_SHIFT, FUNGIBLE_FAUCET, NON_FUNGIBLE_FAUCET,
        REGULAR_ACCOUNT_IMMUTABLE_CODE, REGULAR_ACCOUNT_UPDATABLE_CODE,
    };

    #[test]
    fn test_account_id() {
        use crate::accounts::AccountId;

        for account_type in [
            AccountType::RegularAccountImmutableCode,
            AccountType::RegularAccountUpdatableCode,
            AccountType::NonFungibleFaucet,
            AccountType::FungibleFaucet,
        ] {
            for storage_type in [AccountStorageType::OnChain, AccountStorageType::OffChain] {
                let acc = AccountId::try_from(account_id(account_type, storage_type, 0b1111_1111))
                    .unwrap();
                assert_eq!(acc.account_type(), account_type);
                assert_eq!(acc.storage_type(), storage_type);
            }
        }
    }

    #[test]
    fn test_account_id_from_hex_and_back() {
        for account_id in [
            ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN,
            ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN,
            ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN,
        ] {
            let acc = AccountId::try_from(account_id).expect("Valid account ID");
            assert_eq!(acc, AccountId::from_hex(&acc.to_hex()).unwrap());
        }
    }

    #[test]
    fn test_account_id_serde() {
        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN)
            .expect("Valid account ID");
        assert_eq!(account_id, AccountId::read_from_bytes(&account_id.to_bytes()).unwrap());

        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN)
            .expect("Valid account ID");
        assert_eq!(account_id, AccountId::read_from_bytes(&account_id.to_bytes()).unwrap());

        let account_id =
            AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).expect("Valid account ID");
        assert_eq!(account_id, AccountId::read_from_bytes(&account_id.to_bytes()).unwrap());

        let account_id = AccountId::try_from(ACCOUNT_ID_NON_FUNGIBLE_FAUCET_OFF_CHAIN)
            .expect("Valid account ID");
        assert_eq!(account_id, AccountId::read_from_bytes(&account_id.to_bytes()).unwrap());
    }

    #[test]
    fn test_account_id_account_type() {
        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN)
            .expect("Valid account ID");

        let account_type: AccountType = ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN.into();
        assert_eq!(account_type, account_id.account_type());

        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN)
            .expect("Valid account ID");
        let account_type: AccountType = ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN.into();
        assert_eq!(account_type, account_id.account_type());

        let account_id =
            AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).expect("Valid account ID");
        let account_type: AccountType = ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN.into();
        assert_eq!(account_type, account_id.account_type());

        let account_id = AccountId::try_from(ACCOUNT_ID_NON_FUNGIBLE_FAUCET_OFF_CHAIN)
            .expect("Valid account ID");
        let account_type: AccountType = ACCOUNT_ID_NON_FUNGIBLE_FAUCET_OFF_CHAIN.into();
        assert_eq!(account_type, account_id.account_type());
    }

    #[test]
    fn test_account_id_tag_identifiers() {
        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN)
            .expect("Valid account ID");
        assert!(account_id.is_regular_account());
        assert_eq!(account_id.account_type(), AccountType::RegularAccountImmutableCode);
        assert!(account_id.is_on_chain());

        let account_id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN)
            .expect("Valid account ID");
        assert!(account_id.is_regular_account());
        assert_eq!(account_id.account_type(), AccountType::RegularAccountUpdatableCode);
        assert!(!account_id.is_on_chain());

        let account_id =
            AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).expect("Valid account ID");
        assert!(account_id.is_faucet());
        assert_eq!(account_id.account_type(), AccountType::FungibleFaucet);
        assert!(account_id.is_on_chain());

        let account_id = AccountId::try_from(ACCOUNT_ID_NON_FUNGIBLE_FAUCET_OFF_CHAIN)
            .expect("Valid account ID");
        assert!(account_id.is_faucet());
        assert_eq!(account_id.account_type(), AccountType::NonFungibleFaucet);
        assert!(!account_id.is_on_chain());
    }

    /// The following test ensure there is a bit available to identify an account as a faucet or
    /// normal.
    #[test]
    fn test_account_id_faucet_bit() {
        // faucets have a bit set
        assert_ne!((FUNGIBLE_FAUCET << ACCOUNT_TYPE_MASK_SHIFT) & ACCOUNT_ISFAUCET_MASK, 0);
        assert_ne!((NON_FUNGIBLE_FAUCET << ACCOUNT_TYPE_MASK_SHIFT) & ACCOUNT_ISFAUCET_MASK, 0);

        // normal accounts do not have the faucet bit set
        assert_eq!(
            (REGULAR_ACCOUNT_IMMUTABLE_CODE << ACCOUNT_TYPE_MASK_SHIFT) & ACCOUNT_ISFAUCET_MASK,
            0
        );
        assert_eq!(
            (REGULAR_ACCOUNT_UPDATABLE_CODE << ACCOUNT_TYPE_MASK_SHIFT) & ACCOUNT_ISFAUCET_MASK,
            0
        );
    }
}
