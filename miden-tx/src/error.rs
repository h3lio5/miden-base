use alloc::string::String;
use core::{
    fmt::{self, Display},
    stringify,
};

use miden_objects::{
    assembly::AssemblyError, notes::NoteId, Felt, NoteError, ProvenTransactionError,
    TransactionInputError, TransactionOutputError,
};
use miden_verifier::VerificationError;

use super::{AccountError, AccountId, Digest, ExecutionError};

// TRANSACTION COMPILER ERROR
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionCompilerError {
    AccountInterfaceNotFound(AccountId),
    BuildCodeBlockTableFailed(AssemblyError),
    CompileNoteScriptFailed(AssemblyError),
    CompileTxScriptFailed(AssemblyError),
    LoadAccountFailed(AccountError),
    NoteIncompatibleWithAccountInterface(Digest),
    NoteScriptError(NoteError),
    NoTransactionDriver,
    TxScriptIncompatibleWithAccountInterface(Digest),
}

impl fmt::Display for TransactionCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionCompilerError {}

// TRANSACTION EXECUTOR ERROR
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExecutorError {
    CompileNoteScriptFailed(TransactionCompilerError),
    CompileTransactionScriptFailed(TransactionCompilerError),
    CompileTransactionFailed(TransactionCompilerError),
    ExecuteTransactionProgramFailed(ExecutionError),
    FetchAccountCodeFailed(DataStoreError),
    FetchTransactionInputsFailed(DataStoreError),
    InconsistentAccountId {
        input_id: AccountId,
        output_id: AccountId,
    },
    InconsistentAccountNonceDelta {
        expected: Option<Felt>,
        actual: Option<Felt>,
    },
    InvalidTransactionOutput(TransactionOutputError),
    LoadAccountFailed(TransactionCompilerError),
}

impl fmt::Display for TransactionExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionExecutorError {}

// TRANSACTION PROVER ERROR
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionProverError {
    ProveTransactionProgramFailed(ExecutionError),
    InvalidAccountDelta(AccountError),
    InvalidTransactionOutput(TransactionOutputError),
    ProvenTransactionError(ProvenTransactionError),
}

impl Display for TransactionProverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionProverError::ProveTransactionProgramFailed(inner) => {
                write!(f, "Proving transaction failed: {}", inner)
            },
            TransactionProverError::InvalidAccountDelta(account_error) => {
                write!(f, "Applying account delta failed: {}", account_error)
            },
            TransactionProverError::InvalidTransactionOutput(inner) => {
                write!(f, "Transaction ouptut invalid: {}", inner)
            },
            TransactionProverError::ProvenTransactionError(inner) => {
                write!(f, "Building proven transaction error: {}", inner)
            },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionProverError {}

// TRANSACTION VERIFIER ERROR
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionVerifierError {
    TransactionVerificationFailed(VerificationError),
    InsufficientProofSecurityLevel(u32, u32),
}

impl fmt::Display for TransactionVerifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransactionVerifierError {}

// DATA STORE ERROR
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataStoreError {
    AccountNotFound(AccountId),
    BlockNotFound(u32),
    InvalidTransactionInput(TransactionInputError),
    InternalError(String),
    NoteAlreadyConsumed(NoteId),
    NoteNotFound(NoteId),
}

impl fmt::Display for DataStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DataStoreError {}

// AUTHENTICATION ERROR
// ================================================================================================

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AuthenticationError {
    InternalError(String),
    RejectedSignature(String),
    UnknownKey(String),
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthenticationError::InternalError(error) => {
                write!(f, "authentication internal error: {error}")
            },
            AuthenticationError::RejectedSignature(reason) => {
                write!(f, "signature was rejected: {reason}")
            },
            AuthenticationError::UnknownKey(error) => write!(f, "unknown key error: {error}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AuthenticationError {}

// KERNEL ASSERTION ERRORS
// ================================================================================================

// --- ACCOUNT ERRORS -----------------------------------------------------------------------------    old d  old hex    new d  new hex
const ERR_ACCOUNT_CODE_MUST_BE_UPDATABLE: u32 = 131133;                                             // 131133 0x0002003D 131072 0x00020000
const ERR_ACCOUNT_ID_INSUFFICIENT_NUMBER_OF_ONES: u32 = 131132;                                     // 131132 0x0002003C 
const ERR_ACCOUNT_MUST_BE_A_FAUCET_TO_CALL_PROCEDURE: u32 = 131073;                                 // 131073 0x00020001
const ERR_ACCOUNT_NONCE_INCREASE_MUST_BE_U32_VALUE: u32 = 131131;                                   // 131131 0x0002003B
const ERR_ACCOUNT_POW_IS_INSUFFICIENT: u32 = 131135;                                                // 131135 0x0002003F
const ERR_ACCOUNT_SEED_DIGEST_MISMATCH: u32 = 131134;                                               // 131134 0x0002003E

// --- NONCE ERROR --------------------------------------------------------------------------------
const ERR_NONCE_DID_NOT_INCREASE_AFTER_STATE_CHANGED: u32 = 131081;                                 // 131081 0x00020009

// --- VAULT ERRORS -------------------------------------------------------------------------------
const ERR_VAULT_ADDING_FUNGIBLE_ASSET_WOULD_EXCEED_MAX_AMOUNT_OF_9223372036854775807: u32 = 131117; // 131117 0x0002002D
const ERR_VAULT_ADD_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID: u32 = 131118;           // 131118 0x0002002E
const ERR_VAULT_GET_BALANCE_PROC_CAN_BE_CALLED_ONLY_WITH_FUNGIBLE_FAUCET: u32 = 131115;             // 131115 0x0002002B
const ERR_VAULT_HAS_NON_FUNGIBLE_ASSET_PROC_CAN_BE_CALLED_ONLY_WITH_NON_FUNGIBLE_ASSET: u32 =       // 131116 0x0002002C
    131116;
const ERR_VAULT_NON_FUNGIBLE_ASSET_ALREADY_EXISTS: u32 = 131119;                                    // 131119 0x0002002F
const ERR_VAULT_REMOVE_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID: u32 = 131121;        // 131121 0x00020031
const ERR_VAULT_REMOVING_FUNGIBLE_ASSET_RESULTS_IN_UNDERFLOW: u32 = 131120;                         // 131120 0x00020030
const ERR_VAULT_REMOVING_INEXISTENT_NON_FUNGIBLE_ASSET: u32 = 131122;                               // 131122 0x00020032

// --- FAUCET ERRORS ------------------------------------------------------------------------------
const ERR_FAUCET_BURN_CALLED_ON_NONEXISTENT_TOKEN: u32 = 131110;                                    // 131110 0x00020026
const ERR_FAUCET_BURN_CANNOT_EXCEED_EXISTING_SUPPLY: u32 = 131107;                                  // 131107 0x00020023
const ERR_FAUCET_MINT_WOULD_CAUSE_ISSUANCE_OVERFLOW: u32 = 131106;                                  // 131106 0x00020022
const ERR_FAUCET_NON_FUNGIBLE_BURN_CALLED_ON_WRONG_FAUCET_TYPE: u32 = 131109;                       // 131109 0x00020025
const ERR_FAUCET_NON_FUNGIBLE_TOKEN_ALREADY_EXISTS: u32 = 131108;                                   // 131108 0x00020024
const ERR_FAUCET_STORAGE_DATA_SLOT_254_IS_RESERVED: u32 = 131072;                                   // 131072 0x00020000

// --- ASSET GENERAL ERROR ------------------------------------------------------------------------
const ERR_ASSET_EXCEED_MAX_AMOUNT_OF_9223372036854775807: u32 = 131138;                             // 131138 0x00020042 ! NOT A KERNEL ERROR ?ASSET ERROR?

// --- FUNGIBLE ASSET ERRORS ----------------------------------------------------------------------
const ERR_FUNGIBLE_ASSET_DISTRIBUTE_WOULD_CAUSE_MAX_SUPPLY_TO_BE_EXCEEDED: u32 = 131105;            // 131105 0x00020021 ! NOT A KERNEL ERROR ?FAUCET ERROR?
const ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_ZERO: u32 = 131123;                            // 131123 0x00020033
const ERR_FUNGIBLE_ASSET_FORMAT_POSITION_THREE_MUST_BE_FUNGIBLE_FAUCET_ID: u32 = 131125;            // 131125 0x00020035
const ERR_FUNGIBLE_ASSET_FORMAT_POSITION_TWO_MUST_BE_ZERO: u32 = 131124;                            // 131124 0x00020034
const ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ZERO_EXCEEDS_MAXIMUM_ALLOWED_AMOUNT: u32 = 131126;         // 131126 0x00020036
const ERR_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED: u32 = 131129;                                    // 131129 0x00020039
const ERR_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID: u32 = 131137;                                      // 131137 0x00020041

// --- NON-FUNGIBLE ASSET ERRORS ------------------------------------------------------------------
const ERR_NON_FUNGIBLE_ASSET_FORMAT_HIGH_BIT_MUST_BE_ZERO: u32 = 131128;                            // 131128 0x00020038
const ERR_NON_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_NON_FUNGIBLE_FAUCET_ID: u32 = 131127;      // 131127 0x00020037
const ERR_NON_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED: u32 = 131130;                                // 131130 0x0002003A
const ERR_NON_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID: u32 = 131139;                                  // 131139 0x00020043

// --- NOTE ERRORS --------------------------------------------------------------------------------
const ERR_NOTE_ASSETS_EXCEED_LIMIT_OF_255: u32 = 131114;                                            // 131114 0x0002002A
const ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_ASSETS_FROM_INCORRECT_CONTEXT: u32 = 131112;                  // 131112 0x00020028
const ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_INPUTS_FROM_INCORRECT_CONTEXT: u32 = 131113;                  // 131113 0x00020029
const ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_SENDER_FROM_INCORRECT_CONTEXT: u32 = 131111;                  // 131111 0x00020027
const ERR_NOTE_DATA_DOES_NOT_MATCH_COMMITMENT: u32 = 131136;                                        // 131136 0x00020040
const ERR_NOTE_INVALID_TYPE: u32 = 131140;                                                          // 131140 0x00020044
const ERR_NOTE_TAG_HIGH_BITS_MUST_BE_ZERO: u32 = 131142;                                            // 131142 0x00020046
const ERR_NOTE_TYPE_TAG_PREFIX_IS_INVALID: u32 = 131141;                                            // 131141 0x00020045

// --- PROLOGUE ERRORS ----------------------------------------------------------------------------
const ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_EXCEEDS_256_ELEMENTS: u32 = 131085;                         // 131085 0x0002000D
const ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_INVALID_TYPE_DISCRIMINANT: u32 = 131086;                    // 131086 0x0002000E
const ERR_PROLOGUE_ACCOUNT_STORAGE_DATA_DONT_MATCH_ITS_COMMITMENT: u32 = 131084;                    // 131084 0x0002000C
const ERR_PROLOGUE_MISMATCH_OF_INPUT_NOTES_COMMITMENT_FROM_ADVICE_DATA_AND_KERNEL_INPUTS: u32 =     // 131103 0x0002001F
    131103;
const ERR_PROLOGUE_EXISTING_ACCOUNT_MUST_HAVE_NON_ZERO_NONCE: u32 = 131096;                         // 131096 0x00020018
const ERR_PROLOGUE_GLOBAL_INPUTS_PROVIDED_DONT_MATCH_BLOCK_HASH_COMMITMENT: u32 = 131083;           // 131083 0x0002000B
const ERR_PROLOGUE_MISMATCH_OF_ACCOUNT_IDS_FROM_GLOBAL_INPUTS_AND_ADVICE_PROVIDER: u32 = 131097;    // 131097 0x00020019
const ERR_PROLOGUE_MISMATCH_OF_REFERENCE_BLOCK_MMR_AND_NOTE_AUTHENTICATION_MMR: u32 = 131098;       // 131098 0x0002001A
const ERR_PROLOGUE_NEW_ACCOUNT_SLOT_TYPES_MUST_BE_VALID: u32 = 131088;                              // 131088 0x00020010
const ERR_PROLOGUE_NEW_ACCOUNT_VAULT_MUST_BE_EMPTY: u32 = 131087;                                   // 131087 0x0002000F
const ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE: u32 = 131091;                    // 131091 0x00020013
const ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_EMPTY: u32 = 131089;                   // 131089 0x00020011
const ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY: u32 = 131090;            // 131090 0x00020012
const ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE: u32 = 131094;                // 131094 0x00020016
const ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_VALID_EMPY_SMT: u32 = 131092;      // 131092 0x00020014
const ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY: u32 = 131093;        // 131093 0x00020015
const ERR_PROLOGUE_NUMBER_OF_INPUT_NOTES_EXCEEDED_KERNEL_LIMIT_OF_1023: u32 = 131102;               // 131102 0x0002001E
const ERR_PROLOGUE_NUMBER_OF_NOTE_ASSETS_EXCEEDED_LIMIT_OF_256: u32 = 131100;                       // 131100 0x0002001C
const ERR_PROLOGUE_NUMBER_OF_NOTE_INPUTS_EXCEEDED_LIMIT_OF_128: u32 = 131099;                       // 131099 0x0002001B
const ERR_PROLOGUE_PROVIDED_ACCOUNT_DATA_DONT_MATCH_ON_CHAIN_COMMITMENT: u32 = 131095;              // 131095 0x00020017
const ERR_PROLOGUE_PROVIDED_INPUT_ASSETS_INFO_DONT_MATCH_ITS_COMMITMENT: u32 = 131101;              // 131101 0x0002001D

// --- EPILOGUE ERROR -----------------------------------------------------------------------------
const ERR_EPILOGUE_TOTAL_NUMBER_OF_ASSETS_MUST_STAY_THE_SAME: u32 = 131082;                         // 131082 0x0002000A

// --- TRANSACTION OUTPUT ERROR -------------------------------------------------------------------
const ERR_TX_NUMBER_OF_OUTPUT_NOTES_EXCEEDED_LIMIT_OF_4096: u32 = 131104;                           // 131104 00020020

// NOTE SCRIPT ASSERTION ERRORS
// ================================================================================================

// --- P2ID ERRORS --------------------------------------------------------------------------------
const ERR_P2ID_EXPECTS_EXACTLY_1_NOTE_INPUT: u32 = 131074;                                          // 131074 0x00020002 196608 0x00030000
const ERR_P2ID_MISMATCH_OF_TARGET_ACCOUNT_ADDR_AND_TRANSACTION_ADDR: u32 = 131075;                  // 131075 0x00020003

// --- P2IDR ERRORS -------------------------------------------------------------------------------
const ERR_P2IDR_CAN_BE_RECLAIMED_ONLY_BY_SENDER: u32 = 131077;                                      // 131077 0x00020005
const ERR_P2IDR_EXPECTS_EXACTLY_2_NOTE_INPUTS: u32 = 131076;                                        // 131076 0x00020004
const ERR_P2IDR_RECLAIM_BLOCK_HEIGHT_NOT_REACHED: u32 = 131078;                                     // 131078 0x00020006

// --- SWAP ERRORS --------------------------------------------------------------------------------
const ERR_SWAP_EXPECTS_EXACTLY_9_NOTE_INPUTS: u32 = 131079;                                         // 131079 0x00020007
const ERR_SWAP_REQUIRES_EXACTLY_1_NOTE_ASSET: u32 = 131080;                                         // 131080 0x00020008

#[rustfmt::skip]
pub const ERROR_MESSAGES: [(u32, &str); 71] = [
    (ERR_ACCOUNT_CODE_MUST_BE_UPDATABLE, stringify!(ERR_ACCOUNT_CODE_MUST_BE_UPDATABLE)),
    (ERR_ACCOUNT_ID_INSUFFICIENT_NUMBER_OF_ONES, 
        stringify!(ERR_ACCOUNT_ID_INVALID_INSUFFICIENT_NUMBER_OF_ONES)),
    (ERR_ACCOUNT_MUST_BE_A_FAUCET_TO_CALL_PROCEDURE, 
        stringify!(ERR_ACCOUNT_MUST_BE_A_FAUCET_TO_CALL_PROCEDURE)),
    (ERR_ACCOUNT_NONCE_INCREASE_MUST_BE_U32_VALUE, 
        stringify!(ERR_ACCOUNT_NONCE_INCREASE_MUST_BE_U32_VALUE)),
    (ERR_ACCOUNT_POW_IS_INSUFFICIENT, stringify!(ERR_ACCOUNT_POW_IS_INSUFFICIENT)),
    (ERR_ACCOUNT_SEED_DIGEST_MISMATCH, stringify!(ERR_ACCOUNT_SEED_DIGEST_MISMATCH)),
    
    (ERR_NONCE_DID_NOT_INCREASE_AFTER_STATE_CHANGED, 
        stringify!(ERR_NONCE_DID_NOT_INCREASE_AFTER_STATE_CHANGED)),

    (ERR_VAULT_ADDING_FUNGIBLE_ASSET_WOULD_EXCEED_MAX_AMOUNT_OF_9223372036854775807, 
        stringify!(ERR_VAULT_ADDING_FUNGIBLE_ASSET_WOULD_EXCEED_MAX_AMOUNT_OF_9223372036854775807)),
    (ERR_VAULT_ADD_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID, 
        stringify!(ERR_VAULT_ADD_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID)),
    (ERR_VAULT_GET_BALANCE_PROC_CAN_BE_CALLED_ONLY_WITH_FUNGIBLE_FAUCET, 
        stringify!(ERR_VALUT_GET_BALANCE_PROC_CAN_BE_CALLED_ONLY_WITH_FUNGIBLE_FAUCET)),
    (ERR_VAULT_HAS_NON_FUNGIBLE_ASSET_PROC_CAN_BE_CALLED_ONLY_WITH_NON_FUNGIBLE_ASSET, 
        stringify!(ERR_VAULT_HAS_NON_FUNGIBLE_ASSET_PROC_CAN_BE_CALLED_ONLY_WITH_NON_FUNGIBLE_ASSET)),
    (ERR_VAULT_NON_FUNGIBLE_ASSET_ALREADY_EXISTS, 
        stringify!(ERR_VAULT_NON_FUNGIBLE_ASSET_ALREADY_EXISTS)),
    (ERR_VAULT_REMOVE_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID, 
        stringify!(ERR_VAULT_REMOVE_FUNGIBLE_ASSET_TO_ACCOUNT_FAILED_INITIAL_VALUE_INVALID)),
    (ERR_VAULT_REMOVING_FUNGIBLE_ASSET_RESULTS_IN_UNDERFLOW, 
        stringify!(ERR_VAULT_REMOVING_FUNGIBLE_ASSET_RESULTS_IN_UNDERFLOW)),
    (ERR_VAULT_REMOVING_INEXISTENT_NON_FUNGIBLE_ASSET, 
        stringify!(ERR_VAULT_REMOVING_INEXISTENT_NON_FUNGIBLE_ASSET)),

    (ERR_FAUCET_BURN_CALLED_ON_NONEXISTENT_TOKEN, 
        stringify!(ERR_FAUCET_BURN_CALLED_ON_NONEXISTENT_TOKEN)),
    (ERR_FAUCET_BURN_CANNOT_EXCEED_EXISTING_SUPPLY, 
        stringify!(ERR_FAUCET_BURN_CANNOT_EXCEED_EXISTING_SUPPLY)),
    (ERR_FAUCET_MINT_WOULD_CAUSE_ISSUANCE_OVERFLOW, 
        stringify!(ERR_FAUCET_MINT_WOULD_CAUSE_ISSUANCE_OVERFLOW)),
    (ERR_FAUCET_NON_FUNGIBLE_BURN_CALLED_ON_WRONG_FAUCET_TYPE, 
        stringify!(ERR_FAUCET_NON_FUNGIBLE_BURN_CALLED_ON_WRONG_FAUCET_TYPE)),
    (ERR_FAUCET_NON_FUNGIBLE_TOKEN_ALREADY_EXISTS, 
        stringify!(ERR_FAUCET_NON_FUNGIBLE_TOKEN_ALREADY_EXISTS)),
    (ERR_FAUCET_STORAGE_DATA_SLOT_254_IS_RESERVED, 
        stringify!(ERR_FAUCET_STORAGE_DATA_SLOT_254_IS_RESERVED)),

    (ERR_ASSET_EXCEED_MAX_AMOUNT_OF_9223372036854775807, 
        stringify!(ERR_ASSET_EXCEED_MAX_AMOUNT_OF_9223372036854775807)),

    (ERR_FUNGIBLE_ASSET_DISTRIBUTE_WOULD_CAUSE_MAX_SUPPLY_TO_BE_EXCEEDED, 
        stringify!(ERR_FUNGIBLE_ASSET_DISTRIBUTE_WOULD_CAUSE_MAX_SUPPLY_TO_BE_EXCEEDED)),
    (ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_ZERO, 
        stringify!(ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_ZERO)),
    (ERR_FUNGIBLE_ASSET_FORMAT_POSITION_THREE_MUST_BE_FUNGIBLE_FAUCET_ID, 
        stringify!(ERR_FUNGIBLE_ASSET_FORMAT_POSITION_THREE_MUST_BE_FUNGIBLE_FAUCET_ID)),
    (ERR_FUNGIBLE_ASSET_FORMAT_POSITION_TWO_MUST_BE_ZERO, 
        stringify!(ERR_FUNGIBLE_ASSET_FORMAT_POSITION_TWO_MUST_BE_ZERO)),
    (ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ZERO_EXCEEDS_MAXIMUM_ALLOWED_AMOUNT, 
        stringify!(ERR_FUNGIBLE_ASSET_FORMAT_POSITION_ZERO_EXCEEDS_MAXIMUM_ALLOWED_AMOUNT)),
    (ERR_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED, 
        stringify!(ERR_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED)),
    (ERR_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID, 
        stringify!(ERR_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID)),

    (ERR_NON_FUNGIBLE_ASSET_FORMAT_HIGH_BIT_MUST_BE_ZERO, 
        stringify!(ERR_NON_FUNGIBLE_ASSET_FORMAT_HIGH_BIT_MUST_BE_ZERO)),
    (ERR_NON_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_NON_FUNGIBLE_FAUCET_ID, 
        stringify!(ERR_NON_FUNGIBLE_ASSET_FORMAT_POSITION_ONE_MUST_BE_NON_FUNGIBLE_FAUCET_ID)),
    (ERR_NON_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED, 
        stringify!(ERR_NON_FUNGIBLE_ASSET_ORIGIN_VALIDATION_FAILED)),
    (ERR_NON_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID, 
        stringify!(ERR_NON_FUNGIBLE_ASSET_PROVIDED_ID_IS_INVALID)),

    (ERR_NOTE_ASSETS_EXCEED_LIMIT_OF_255, stringify!(ERR_NOTE_ASSETS_EXCEED_LIMIT_OF_255)),
    (ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_ASSETS_FROM_INCORRECT_CONTEXT, 
        stringify!(ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_ASSETS_FROM_INCORRECT_CONTEXT)),
    (ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_INPUTS_FROM_INCORRECT_CONTEXT, 
        stringify!(ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_INPUTS_FROM_INCORRECT_CONTEXT)),
    (ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_SENDER_FROM_INCORRECT_CONTEXT, 
        stringify!(ERR_NOTE_ATTEMPT_TO_ACCESS_NOTE_SENDER_FROM_INCORRECT_CONTEXT)),
    (ERR_NOTE_DATA_DOES_NOT_MATCH_COMMITMENT, stringify!(ERR_NOTE_DATA_DOES_NOT_MATCH_COMMITMENT)),
    (ERR_NOTE_INVALID_TYPE, stringify!(ERR_NOTE_INVALID_TYPE)),
    (ERR_NOTE_TAG_HIGH_BITS_MUST_BE_ZERO, stringify!(ERR_NOTE_TAG_HIGH_BITS_MUST_BE_ZERO)),
    (ERR_NOTE_TYPE_TAG_PREFIX_IS_INVALID, stringify!(ERR_NOTE_TYPE_TAG_PREFIX_IS_INVALID)),

    (ERR_P2ID_EXPECTS_EXACTLY_1_NOTE_INPUT, stringify!(ERR_P2ID_EXPECTS_EXACTLY_1_NOTE_INPUT)),
    (ERR_P2ID_MISMATCH_OF_TARGET_ACCOUNT_ADDR_AND_TRANSACTION_ADDR, 
        stringify!(ERR_P2ID_MISMATCH_OF_TARGET_ACCOUNT_ADDR_AND_TRANSACTION_ADDR)),
    
    (ERR_P2IDR_CAN_BE_RECLAIMED_ONLY_BY_SENDER, 
        stringify!(ERR_P2IDR_CAN_BE_RECLAIMED_ONLY_BY_SENDER)),
    (ERR_P2IDR_EXPECTS_EXACTLY_2_NOTE_INPUTS, stringify!(ERR_P2IDR_EXPECTS_EXACTLY_2_NOTE_INPUTS)),
    (ERR_P2IDR_RECLAIM_BLOCK_HEIGHT_NOT_REACHED, 
        stringify!(ERR_P2IDR_RECLAIM_BLOCK_HEIGHT_NOT_REACHED)),

    (ERR_SWAP_EXPECTS_EXACTLY_9_NOTE_INPUTS, stringify!(ERR_SWAP_EXPECTS_EXACTLY_9_NOTE_INPUTS)),
    (ERR_SWAP_REQUIRES_EXACTLY_1_NOTE_ASSET, stringify!(ERR_SWAP_REQUIRES_EXACTLY_1_NOTE_ASSET)),

    (ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_EXCEEDS_256_ELEMENTS, 
        stringify!(ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_EXCEEDS_256_ELEMENTS)),
    (ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_INVALID_TYPE_DISCRIMINANT, 
        stringify!(ERR_PROLOGUE_ACCOUNT_DATA_STORAGE_INVALID_TYPE_DISCRIMINANT)),
    (ERR_PROLOGUE_ACCOUNT_STORAGE_DATA_DONT_MATCH_ITS_COMMITMENT, 
        stringify!(ERR_PROLOGUE_ACCOUNT_STORAGE_DATA_DONT_MATCH_ITS_COMMITMENT)),
    (ERR_PROLOGUE_MISMATCH_OF_INPUT_NOTES_COMMITMENT_FROM_ADVICE_DATA_AND_KERNEL_INPUTS, 
        stringify!(ERR_PROLOGUE_MISMATCH_OF_INPUT_NOTES_COMMITMENT_FROM_ADVICE_DATA_AND_KERNEL_INPUTS)),
    (ERR_PROLOGUE_EXISTING_ACCOUNT_MUST_HAVE_NON_ZERO_NONCE, 
        stringify!(ERR_PROLOGUE_EXISTING_ACCOUNT_MUST_HAVE_NON_ZERO_NONCE)),
    (ERR_PROLOGUE_GLOBAL_INPUTS_PROVIDED_DONT_MATCH_BLOCK_HASH_COMMITMENT, 
        stringify!(ERR_PROLOGUE_GLOBAL_INPUTS_PROVIDED_DONT_MATCH_BLOCK_HASH_COMMITMENT)),
    (ERR_PROLOGUE_MISMATCH_OF_ACCOUNT_IDS_FROM_GLOBAL_INPUTS_AND_ADVICE_PROVIDER, 
        stringify!(ERR_PROLOGUE_MISMATCH_OF_ACCOUNT_IDS_FROM_GLOBAL_INPUTS_AND_ADVICE_PROVIDER)),
    (ERR_PROLOGUE_MISMATCH_OF_REFERENCE_BLOCK_MMR_AND_NOTE_AUTHENTICATION_MMR, 
        stringify!(ERR_PROLOGUE_MISMATCH_OF_REFERENCE_BLOCK_MMR_AND_NOTE_AUTHENTICATION_MMR)),
    (ERR_PROLOGUE_NEW_ACCOUNT_SLOT_TYPES_MUST_BE_VALID, 
        stringify!(ERR_PROLOGUE_NEW_ACCOUNT_SLOT_TYPES_MUST_BE_VALID)),
    (ERR_PROLOGUE_NEW_ACCOUNT_VAULT_MUST_BE_EMPTY, 
        stringify!(ERR_PROLOGUE_NEW_ACCOUNT_VAULT_MUST_BE_EMPTY)),
    (ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE, 
        stringify!(ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE)),
    (ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_EMPTY, 
        stringify!(ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_EMPTY)),
    (ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY, 
        stringify!(ERR_PROLOGUE_NEW_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY)),
    (ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE, 
        stringify!(ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_INVALID_TYPE)),
    (ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_VALID_EMPY_SMT, 
        stringify!(ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_BE_VALID_EMPY_SMT)),
    (ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY, 
        stringify!(ERR_PROLOGUE_NEW_NON_FUNGIBLE_FAUCET_RESERVED_SLOT_MUST_HAVE_ZERO_ARITY)),
    (ERR_PROLOGUE_NUMBER_OF_INPUT_NOTES_EXCEEDED_KERNEL_LIMIT_OF_1023, 
        stringify!(ERR_PROLOGUE_NUMBER_OF_INPUT_NOTES_EXCEEDED_KERNEL_LIMIT_OF_1023)),
    (ERR_PROLOGUE_NUMBER_OF_NOTE_ASSETS_EXCEEDED_LIMIT_OF_256, 
        stringify!(ERR_PROLOGUE_NUMBER_OF_NOTE_ASSETS_EXCEEDED_LIMIT_OF_256)),
    (ERR_PROLOGUE_NUMBER_OF_NOTE_INPUTS_EXCEEDED_LIMIT_OF_128, 
        stringify!(ERR_PROLOGUE_NUMBER_OF_NOTE_INPUTS_EXCEEDED_LIMIT_OF_128)),
    (ERR_PROLOGUE_PROVIDED_ACCOUNT_DATA_DONT_MATCH_ON_CHAIN_COMMITMENT, 
        stringify!(ERR_PROLOGUE_PROVIDED_ACCOUNT_DATA_DONT_MATCH_ON_CHAIN_COMMITMENT)),
    (ERR_PROLOGUE_PROVIDED_INPUT_ASSETS_INFO_DONT_MATCH_ITS_COMMITMENT, 
        stringify!(ERR_PROLOGUE_PROVIDED_INPUT_ASSETS_INFO_DONT_MATCH_ITS_COMMITMENT)),

    (ERR_EPILOGUE_TOTAL_NUMBER_OF_ASSETS_MUST_STAY_THE_SAME, 
        stringify!(ERR_EPILOGUE_TOTAL_NUMBER_OF_ASSETS_MUST_STAY_THE_SAME)),

    (ERR_TX_NUMBER_OF_OUTPUT_NOTES_EXCEEDED_LIMIT_OF_4096, 
        stringify!(ERR_TX_NUMBER_OF_OUTPUT_NOTES_EXCEEDED_LIMIT_OF_4096)),
];
