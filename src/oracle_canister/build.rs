use solidity_helper::compile_solidity_contracts;

fn main() {
    let contracts =
        compile_solidity_contracts(None, None).expect("Should compile solidity smart contracts");

    let aggregator_single_contract_hex = &contracts
        .get("AggregatorSingle")
        .expect("Cannot find the AggregatorSingle contract")
        .bytecode_hex;

    set_var(
        "BUILD_SMART_CONTRACT_AGGREGATOR_SINGLE_HEX_CODE",
        aggregator_single_contract_hex,
    );
}

// this sets a compile time variable
fn set_var(key: &str, value: &str) {
    println!("cargo:rustc-env={key}={value}");
}
