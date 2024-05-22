use core::fmt;
use std::{
    fs::{read_to_string, write, File},
    io::Write,
    path::Path,
};

use miden_lib::{
    notes::create_p2id_note, transaction::ToTransactionKernelInputs, utils::Serializable,
};
use miden_objects::{
    accounts::AccountId,
    assembly::ProgramAst,
    assets::{Asset, FungibleAsset},
    crypto::{dsa::rpo_falcon512::SecretKey, rand::RpoRandomCoin},
    notes::NoteType,
    transaction::TransactionArgs,
    Felt,
};
use miden_tx::{
    host::BasicAuthenticator, AuthSecretKey, TransactionExecutor, TransactionHost,
    TransactionProgress,
};
use rand::rngs::StdRng;
use vm_processor::{ExecutionOptions, RecAdviceProvider, Word};

mod utils;
use utils::{
    get_account_with_default_account_code, write_bench_results_to_json, MockDataStore,
    ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN, ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN,
    ACCOUNT_ID_SENDER, DEFAULT_AUTH_SCRIPT,
};
pub enum Benchmark {
    Simple,
    P2ID,
}

impl fmt::Display for Benchmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Benchmark::Simple => write!(f, "simple"),
            Benchmark::P2ID => write!(f, "p2id"),
        }
    }
}

fn main() -> Result<(), String> {
    // create a template file for benchmark results
    let path = Path::new("bench-tx/bench-tx.json");
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(b"{}").map_err(|e| e.to_string())?;

    // run all available benchmarks
    let benchmark_results = vec![
        (Benchmark::Simple, benchmark_default_tx()?),
        (Benchmark::P2ID, benchmark_p2id()?),
    ];

    // store benchmark results in the JSON file
    write_bench_results_to_json(path, benchmark_results)?;

    Ok(())
}

// BENCHMARKS
// ================================================================================================

/// Runs the default transaction with empty transaction script and two default notes.
pub fn benchmark_default_tx() -> Result<TransactionProgress, String> {
    let data_store = MockDataStore::default();
    let mut executor: TransactionExecutor<_, ()> =
        TransactionExecutor::new(data_store.clone(), None).with_tracing();

    let account_id = data_store.account.id();
    executor.load_account(account_id).map_err(|e| e.to_string())?;

    let block_ref = data_store.block_header.block_num();
    let note_ids = data_store.notes.iter().map(|note| note.id()).collect::<Vec<_>>();

    let transaction = executor
        .prepare_transaction(account_id, block_ref, &note_ids, data_store.tx_args().clone())
        .map_err(|e| e.to_string())?;

    let (stack_inputs, advice_inputs) = transaction.get_kernel_inputs();
    let advice_recorder: RecAdviceProvider = advice_inputs.into();
    let mut host: TransactionHost<_, ()> =
        TransactionHost::new(transaction.account().into(), advice_recorder, None);

    vm_processor::execute(
        transaction.program(),
        stack_inputs,
        &mut host,
        ExecutionOptions::default().with_tracing(),
    )
    .map_err(|e| e.to_string())?;

    Ok(host.tx_progress().clone())
}

/// Runs the transaction which consumes a P2ID note into a basic wallet.
pub fn benchmark_p2id() -> Result<TransactionProgress, String> {
    // Create assets
    let faucet_id = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let fungible_asset: Asset = FungibleAsset::new(faucet_id, 100).unwrap().into();

    // Create sender and target account
    let sender_account_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

    let target_account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN).unwrap();
    let sec_key = SecretKey::new();
    let target_pub_key: Word = sec_key.public_key().into();
    let mut pk_sk_bytes = sec_key.to_bytes();
    pk_sk_bytes.append(&mut target_pub_key.to_bytes());
    let target_sk_pk_felt: Vec<Felt> =
        pk_sk_bytes.iter().map(|a| Felt::new(*a as u64)).collect::<Vec<Felt>>();
    let target_account =
        get_account_with_default_account_code(target_account_id, target_pub_key, None);

    // Create the note
    let note = create_p2id_note(
        sender_account_id,
        target_account_id,
        vec![fungible_asset],
        NoteType::Public,
        RpoRandomCoin::new([Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]),
    )
    .unwrap();

    let data_store =
        MockDataStore::with_existing(Some(target_account.clone()), Some(vec![note.clone()]));

    let mut executor: TransactionExecutor<_, ()> =
        TransactionExecutor::new(data_store.clone(), None).with_tracing();
    executor.load_account(target_account_id).unwrap();

    let block_ref = data_store.block_header.block_num();
    let note_ids = data_store.notes.iter().map(|note| note.id()).collect::<Vec<_>>();

    let tx_script_code = ProgramAst::parse(DEFAULT_AUTH_SCRIPT).unwrap();

    let tx_script_target = executor
        .compile_tx_script(
            tx_script_code.clone(),
            vec![(target_pub_key, target_sk_pk_felt)],
            vec![],
        )
        .unwrap();
    let tx_args_target = TransactionArgs::with_tx_script(tx_script_target);

    // execute transaction
    let transaction = executor
        .prepare_transaction(target_account_id, block_ref, &note_ids, tx_args_target)
        .map_err(|e| e.to_string())?;

    let (stack_inputs, advice_inputs) = transaction.get_kernel_inputs();
    let advice_recorder: RecAdviceProvider = advice_inputs.into();
    let authenticator = BasicAuthenticator::<StdRng>::new(&[(
        sec_key.public_key().into(),
        AuthSecretKey::RpoFalcon512(sec_key),
    )]);
    let authenticator = Some(authenticator);
    let mut host =
        TransactionHost::new(transaction.account().into(), advice_recorder, authenticator);

    vm_processor::execute(
        transaction.program(),
        stack_inputs,
        &mut host,
        ExecutionOptions::default().with_tracing(),
    )
    .map_err(|e| e.to_string())?;

    Ok(host.tx_progress().clone())
}
