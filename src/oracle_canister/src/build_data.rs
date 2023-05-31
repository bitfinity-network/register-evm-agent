use crate::error::{Error, Result};

/// AggregatorSingle smart contract hex code
const BUILD_SMART_CONTRACT_AGGREGATOR_SINGLE_HEX_CODE: &str =
    env!("BUILD_SMART_CONTRACT_AGGREGATOR_SINGLE_HEX_CODE");

/// Returns the AggregatorSingle smart contract bytecode
pub fn get_aggregator_single_smart_contract_code() -> Result<Vec<u8>> {
    hex::decode(BUILD_SMART_CONTRACT_AGGREGATOR_SINGLE_HEX_CODE)
        .map_err(|err| Error::Internal(format!("{err:?}")))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_get_aggregator_single_smart_contract_code() {
        let code = get_aggregator_single_smart_contract_code().unwrap();
        assert!(!code.is_empty())
    }
}
