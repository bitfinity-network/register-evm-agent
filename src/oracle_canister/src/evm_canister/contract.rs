use std::borrow::Cow;
use std::cell::RefCell;
use std::default;

use async_trait::async_trait;
use candid::{CandidType, Principal};
use ethers_core::abi::{Constructor, Function, Param, ParamType, StateMutability, Token};
use ethers_core::types::Signature;
use ic_exports::ic_kit::ic;
use ic_stable_structures::{BoundedStorable, StableBTreeMap, StableCell, Storable};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::build_data::get_aggregator_proxy_smart_contract_code;
use crate::error::{Error, Result};
use crate::evm_canister::did::{Transaction, H160, H256, U256, U64};
use crate::evm_canister::EvmCanisterImpl;
use crate::state::TOKENS_REGISTRATION_STATE_MEMORY_ID;

use super::EvmCanister;

#[derive(Debug, PartialEq, Eq, Default)]
enum ContractStatus {
    #[default]
    Unregistered,
    RegistrationInProgress,
    Registered(H160),
}

const UNREGISTERED_DATA: &[u8] = &[0u8; 20];
const REGISTRATION_IN_PROGRESS_DATA: &[u8] = &[1u8; 20];

impl Storable for ContractStatus {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        match &self {
            ContractStatus::Unregistered => Cow::Borrowed(UNREGISTERED_DATA),
            ContractStatus::RegistrationInProgress => Cow::Borrowed(REGISTRATION_IN_PROGRESS_DATA),
            ContractStatus::Registered(hash) => Cow::Borrowed(&(hash.0 .0)),
        }
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        match bytes.as_ref() {
            UNREGISTERED_DATA => ContractStatus::Unregistered,
            REGISTRATION_IN_PROGRESS_DATA => ContractStatus::RegistrationInProgress,
            hash => ContractStatus::Registered(H160::from_slice(hash)),
        }
    }
}

impl BoundedStorable for ContractStatus {
    const MAX_SIZE: u32 = 20;

    const IS_FIXED_SIZE: bool = true;
}

pub struct ContractService {}

impl ContractService {
    pub async fn init_contract(aggregator_address: H160) -> Result<H256> {
        // Check if the contract is already registered or pending
        // Note that there are no await points between this check and create contract
        if CONTRACT_REGISTRATION_STATE.with(|data| {
            if *data.borrow().get() != ContractStatus::Unregistered {
                true
            } else {
                let res = data
                    .borrow_mut()
                    .set(ContractStatus::RegistrationInProgress)
                    .expect("set contract registration in stable memory error");

                false
            }
        }) {
            return Err(Error::ContractAlreadyRegistered);
        }

        let contract = get_aggregator_proxy_smart_contract_code()?;

        let contract_data = AGGREGATOR_PROXY_CONSTRUCTOR
            .encode_input(contract, &[Token::Address(aggregator_address.into())])
            .map_err(|e| {
                Error::Internal(format!("failed to encode contract constructor args: {e:?}"))
            })?;

        let mut evm_impl = EvmCanisterImpl::default();
        let tx_hash = match evm_impl.create_contract(U256::zero(), contract_data).await {
            Ok(hash) => hash,
            Err(err) => {
                CONTRACT_REGISTRATION_STATE.with(|data| {
                    data.borrow_mut()
                        .set(ContractStatus::default())
                        .expect("set contract registration in stable memory error")
                });
                return Err(err);
            }
        };

        Ok(tx_hash)
    }

    /// Call the specified contract function with the given arguments
    async fn call_contract_func(func: &Function, args: &[Token], contract: H160) -> Result<H256> {
        let call_data = func
            .encode_input(args)
            .map_err(|e| Error::Internal(format!("failed to encode solidity call data: {e:?}")))?;

        let mut evm_impl = EvmCanisterImpl::default();

        let tx_hash = evm_impl.transact(U256::zero(), contract, call_data).await?;

        Ok(tx_hash)
    }
}

thread_local! {
    static CONTRACT_REGISTRATION_STATE: RefCell<StableCell<ContractStatus>> =
        RefCell::new(StableCell::new(TOKENS_REGISTRATION_STATE_MEMORY_ID, ContractStatus::default()).expect("init contract registration in stable memory error"));
}

static AGGREGATOR_PROXY_CONSTRUCTOR: Lazy<Constructor> = Lazy::new(|| Constructor {
    inputs: vec![Param {
        name: "aggregatorAddress".into(),
        kind: ParamType::Address,
        internal_type: None,
    }],
});
