use std::borrow::Cow;
use std::cell::RefCell;

use ethers_core::abi::{Constructor, Function, Param, ParamType, StateMutability, Token};
use ic_stable_structures::{BoundedStorable, StableCell, Storable};

use crate::build_data::get_aggregator_single_smart_contract_code;
use crate::error::{Error, Result};
use crate::evm_canister::did::{TransactionReceipt, H160, H256, U256, U64};
use crate::evm_canister::EvmCanisterImpl;
use crate::state::{
    CONTRACT_REGISTRATION_STATE_MEMORY_ID, CONTRACT_REGISTRATION_TX_HASH_MEMORY_ID,
};

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

#[derive(Default, Clone)]
pub struct ContractService {}

impl ContractService {
    // deploy the AggregatorSingle contract to evmc, and stored the tx hash.
    pub async fn init_contract(&mut self) -> Result<H256> {
        // Check if the contract is already registered or pending
        // Note that there are no await points between this check and create contract
        if CONTRACT_REGISTRATION_STATE.with(|data| {
            if *data.borrow().get() != ContractStatus::Unregistered {
                true
            } else {
                data.borrow_mut()
                    .set(ContractStatus::RegistrationInProgress)
                    .expect("set contract registration in stable memory error");

                false
            }
        }) {
            return Err(Error::ContractAlreadyRegistered);
        }

        let contract = get_aggregator_single_smart_contract_code()?;

        let constructor = Constructor { inputs: vec![] };
        let contract_data = constructor.encode_input(contract, &[]).map_err(|e| {
            Error::Internal(format!("failed to encode contract constructor args: {e:?}"))
        })?;

        let mut evm_impl = EvmCanisterImpl::default();
        let tx_hash = match evm_impl.create_contract(U256::zero(), contract_data).await {
            Ok(hash) => {
                CONTRACT_REGISTRATION_TX_HASH.with(|c| {
                    c.borrow_mut()
                        .set(hash.clone())
                        .expect("set contract registration in stable memory error")
                });
                hash
            }
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

    // Make sure the deployment is successful and get the contract address from the transaction receipt
    pub async fn confirm_contract_address(&mut self) -> Result<H160> {
        let hash = CONTRACT_REGISTRATION_TX_HASH
            .with(|c| c.borrow().get().clone())
            .clone();
        if hash == H256::zero() {
            return Err(Error::ContractNotRegistered);
        }

        let evm_impl = EvmCanisterImpl::default();
        let addr_opt = match evm_impl.get_transaction_receipt_by_hash(hash).await {
            Ok(Some(receipt)) => Self::get_created_contract_address(receipt),
            _ => None,
        };

        if let Some(addr) = addr_opt {
            CONTRACT_REGISTRATION_STATE.with(|data| {
                data.borrow_mut()
                    .set(ContractStatus::Registered(addr.clone()))
                    .expect("set CONTRACT_REGISTRATION_STATE error")
            });

            CONTRACT_REGISTRATION_TX_HASH.with(|data| {
                data.borrow_mut()
                    .set(H256::zero())
                    .expect("set CONTRACT_REGISTRATION_TX_HASH error")
            });
            Ok(addr)
        } else {
            // need to check out whether the tx failed or tx in memory pool
            // if tx failed:
            CONTRACT_REGISTRATION_STATE.with(|data| {
                data.borrow_mut()
                    .set(ContractStatus::Unregistered)
                    .expect("set CONTRACT_REGISTRATION_STATE error")
            });
            Err(Error::Internal("evm canister: tx failed.".to_string()))
        }
    }

    /// Call the Aggregator contract in evmc to increase the currency price pairs supported by the aggregator
    #[allow(deprecated)]
    pub async fn add_pair(
        &self,
        pair: String,
        decimal: U256,
        description: String,
        version: U256,
    ) -> Result<H256> {
        let contract = self.get_contract()?;

        let add_pair_func = Function {
            name: "addPair".into(),
            inputs: vec![
                Param {
                    name: "pair".into(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "decimal".into(),
                    kind: ParamType::Uint(8),
                    internal_type: None,
                },
                Param {
                    name: "description".into(),
                    kind: ParamType::String,
                    internal_type: None,
                },
                Param {
                    name: "version".into(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![],
            constant: None,
            state_mutability: StateMutability::NonPayable,
        };
        let args = [
            Token::String(pair),
            Token::Uint(decimal.0),
            Token::String(description),
            Token::Uint(version.0),
        ];

        Self::call_contract_func(&add_pair_func, &args, contract).await
    }

    /// Call the Aggregator contract in evmc to update the supported currency price pairs.
    #[allow(deprecated)]
    pub async fn update_answers(
        &self,
        pairs: Vec<String>,
        timestamps: Vec<U256>,
        prices: Vec<U256>,
    ) -> Result<H256> {
        let contract = self.get_contract()?;

        let add_pair_func = Function {
            name: "updateAnswers".into(),
            inputs: vec![
                Param {
                    name: "_pairs".into(),
                    kind: ParamType::Array(ParamType::String.into()),
                    internal_type: None,
                },
                Param {
                    name: "_timestamps".into(),
                    kind: ParamType::Array(ParamType::Uint(256).into()),
                    internal_type: None,
                },
                Param {
                    name: "_answers".into(),
                    kind: ParamType::Array(ParamType::Uint(256).into()),
                    internal_type: None,
                },
            ],
            outputs: vec![],
            constant: None,
            state_mutability: StateMutability::NonPayable,
        };
        let pairs = pairs.into_iter().map(Token::String).collect();
        let timestamps = timestamps.into_iter().map(|t| Token::Uint(t.0)).collect();
        let prices = prices.into_iter().map(|p| Token::Uint(p.0)).collect();
        let args = [
            Token::Array(pairs),
            Token::Array(timestamps),
            Token::Array(prices),
        ];

        Self::call_contract_func(&add_pair_func, &args, contract).await
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

    pub fn get_contract(&self) -> Result<H160> {
        CONTRACT_REGISTRATION_STATE.with(|c| {
            if let ContractStatus::Registered(contract) = c.borrow().get() {
                Ok(contract.clone())
            } else {
                Err(Error::ContractNotRegistered)
            }
        })
    }

    fn get_created_contract_address(result: TransactionReceipt) -> Option<H160> {
        if Some(U64::one()) == result.status {
            result.contract_address
        } else {
            None
        }
    }
}

thread_local! {
    static CONTRACT_REGISTRATION_STATE: RefCell<StableCell<ContractStatus>> =
        RefCell::new(StableCell::new(CONTRACT_REGISTRATION_STATE_MEMORY_ID, ContractStatus::default()).expect("init contract registration state in stable memory error"));

    static CONTRACT_REGISTRATION_TX_HASH: RefCell<StableCell<H256>> =
        RefCell::new(StableCell::new(CONTRACT_REGISTRATION_TX_HASH_MEMORY_ID, H256::zero()).expect("init contract registration tx hash in stable memory error"));
}
