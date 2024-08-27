use crate::Loader;
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS,
    ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*},
    context::Context,
};

const MAX_CYCLES: u64 = 10_000_000;

// Include your tests here
// See https://github.com/xxuejie/ckb-native-build-sample/blob/main/tests/src/tests.rs for more examples

#[test]
fn test_since_time_lock() {
    let mut context = Context::default();

    // * bin => out_point => Script, cell_dep
    let required_lock_script_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    let required_lock_script = context
        .build_script(&required_lock_script_out_point.clone(), Default::default())
        .expect("script");
    let required_lock_script_cell_dep = CellDep::new_builder()
        .out_point(required_lock_script_out_point.clone())
        .build();

    let locked_until = 4u64;
    let locked_until_bytes = locked_until.to_le_bytes();
    let required_script_hash_bytes32 = required_lock_script.calc_script_hash();

    // concat lock_until and required_lock_script_hash
    let mut since_time_lock_script_args_vec: Vec<u8> = Vec::new();
    since_time_lock_script_args_vec.extend_from_slice(&locked_until_bytes);
    since_time_lock_script_args_vec.extend_from_slice(&required_script_hash_bytes32.as_bytes());
    let since_time_lock_script_args: Bytes = Bytes::from(since_time_lock_script_args_vec);
    let since_time_lock_bin: Bytes = Loader::default().load_binary("since-time-lock");
    let since_time_lock_bin_out_point = context.deploy_cell(since_time_lock_bin);
    let since_time_lock_script = context
        .build_script(
            &since_time_lock_bin_out_point.clone(),
            since_time_lock_script_args,
        )
        .expect("script");
    let since_time_lock_cell_dep = CellDep::new_builder()
        .out_point(since_time_lock_bin_out_point.clone())
        .build();

    let cell_deps: Vec<CellDep> = vec![required_lock_script_cell_dep, since_time_lock_cell_dep];

    // prepare cells
    let cannot_unlock_yet = 2u64;
    let can_unlock = 33u64;
    let utxo_0 = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(since_time_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let cell_input_0 = CellInput::new_builder()
        .previous_output(utxo_0.clone())
        .since(cannot_unlock_yet.pack())
        .build();
    let utxo_1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(required_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let cell_input_1 = CellInput::new_builder()
        .previous_output(utxo_1.clone())
        .since(can_unlock.pack())
        .build();
    let cell_input_2 = CellInput::new_builder()
        .previous_output(utxo_1.clone())
        .since(cannot_unlock_yet.pack())
        .build();

    let outputs = vec![
        CellOutput::new_builder().capacity(500u64.pack()).build(),
        CellOutput::new_builder().capacity(500u64.pack()).build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    let cell_inputs_missing_required_lock: Vec<CellInput> = vec![cell_input_0.clone()];
    let cell_inputs_cannot_unlock_yet: Vec<CellInput> =
        vec![cell_input_0.clone(), cell_input_2.clone()];
    let cell_inputs_ok: Vec<CellInput> = vec![cell_input_0.clone(), cell_input_1.clone()];

    // build transaction
    let tx_missing_required_lock = TransactionBuilder::default()
        .cell_deps(cell_deps.clone())
        .inputs(cell_inputs_missing_required_lock)
        .outputs(outputs.clone())
        .outputs_data(outputs_data.clone().pack())
        .build();
    let tx_missing_required_lock = tx_missing_required_lock.as_advanced_builder().build();
    context
        .verify_tx(&tx_missing_required_lock, MAX_CYCLES)
        .expect_err("cannot unlock without required lock script");

    let tx_cannot_unlock_yet = TransactionBuilder::default()
        .cell_deps(cell_deps.clone())
        .inputs(cell_inputs_cannot_unlock_yet)
        .outputs(outputs.clone())
        .outputs_data(outputs_data.clone().pack())
        .build();
    let tx_cannot_unlock_yet = tx_cannot_unlock_yet.as_advanced_builder().build();
    context
        .verify_tx(&tx_cannot_unlock_yet, MAX_CYCLES)
        .expect_err("cannot unlock yet");

    let tx_ok = TransactionBuilder::default()
        .cell_deps(cell_deps)
        .inputs(cell_inputs_ok)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();

    let tx_ok = tx_ok.as_advanced_builder().build();
    let cycles = context
        .verify_tx(&tx_ok, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
