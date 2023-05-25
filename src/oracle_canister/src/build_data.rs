use crate::error::{Error, Result};

/// AggregatorProxy smart contract hex code
const BUILD_SMART_CONTRACT_AGGREGATOR_PROXY_HEX_CODE: &str =
    env!("BUILD_SMART_CONTRACT_AGGREGATOR_PROXY_HEX_CODE");

/// Returns the AggregatorProxy smart contract bytecode
pub fn get_aggregator_proxy_smart_contract_code() -> Result<Vec<u8>> {
    hex::decode(BUILD_SMART_CONTRACT_AGGREGATOR_PROXY_HEX_CODE)
        .map_err(|err| Error::Internal(format!("{err:?}")))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_get_aggregator_proxy_smart_contract_code() {
        let code = get_aggregator_proxy_smart_contract_code().unwrap();
        assert!(!code.is_empty())
    }
}
