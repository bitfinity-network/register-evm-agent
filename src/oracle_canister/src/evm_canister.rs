use std::cell::RefCell;

use async_trait::async_trait;
use candid::Principal;
use ic_exports::ic_kit::ic;
use ic_exports::ic_kit::RejectionCode;
use ic_stable_structures::StableCell;
use mockall::automock;

use crate::error::Error;
use crate::evm_canister::{
    did::{BasicAccount, Transaction, TransactionParams, TransactionReceipt, H160, H256, U256},
    error::{EvmError, TransactionPoolError},
};
use crate::state::{State, NONCE_MEMORY_ID};

pub mod account;
pub mod contract;
pub mod did;
pub mod error;

pub const REGISTRATION_FEE: u64 = 100_000;
pub const DEFAULT_GAS_LIMIT: u64 = 30_000_000;

type EvmResult<T> = Result<T, EvmError>;

/// Interface for calling EVMC methods
#[automock]
#[async_trait(?Send)]
pub trait EvmCanister: Send {
    async fn transact(&mut self, value: U256, to: H160, data: Vec<u8>) -> Result<H256, Error>;

    async fn create_contract(&mut self, value: U256, code: Vec<u8>) -> Result<H256, Error>;

    async fn get_balance(&self, address: H160) -> Result<U256, Error>;

    async fn get_transaction_by_hash(&self, tx_hash: H256) -> Result<Option<Transaction>, Error>;

    async fn get_transaction_receipt_by_hash(
        &self,
        tx_hash: H256,
    ) -> Result<Option<TransactionReceipt>, Error>;

    async fn mint_evm_tokens(&mut self, to: H160, amount: U256) -> Result<U256, Error>;

    async fn register_ic_agent(&mut self, transaction: Transaction) -> Result<(), Error>;

    async fn verify_registration(&mut self, signing_key: Vec<u8>) -> Result<(), Error>;

    async fn is_address_registered(&self, address: H160) -> Result<bool, Error>;
}

#[derive(Default)]
pub struct EvmCanisterImpl {}

impl EvmCanisterImpl {
    fn get_evm_canister_id(&self) -> Principal {
        State::default().config.get_evmc_principal()
    }

    fn get_nonce(&self) -> U256 {
        NONCE_CELL.with(|nonce| {
            let value = nonce.borrow().get().clone();
            nonce
                .borrow_mut()
                .set(value.clone() + U256::one())
                .expect("failed to update nonce");
            value
        })
    }

    fn process_call<T>(
        &self,
        result: Result<T, (RejectionCode, std::string::String)>,
    ) -> Result<T, Error> {
        result.map_err(|e| Error::Internal(format!("ic call failure: {e:?}")))
    }

    fn process_call_result<T>(
        &self,
        result: Result<EvmResult<T>, (RejectionCode, std::string::String)>,
    ) -> Result<T, Error> {
        let result = self.process_call(result)?;
        if let Err(EvmError::TransactionPool(TransactionPoolError::InvalidNonce {
            expected, ..
        })) = &result
        {
            NONCE_CELL.with(|nonce| {
                nonce
                    .borrow_mut()
                    .set(expected.clone())
                    .expect("failed to update nonce");
            });
        }

        result.map_err(|e| Error::Internal(format!("transaction error: {e}")))
    }

    fn get_tx_params(&self, value: U256) -> Result<TransactionParams, Error> {
        Ok(TransactionParams {
            from: account::Account::default().get_account()?,
            value,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_price: None,
            nonce: self.get_nonce(),
        })
    }
}

#[async_trait(?Send)]
impl EvmCanister for EvmCanisterImpl {
    async fn transact(&mut self, value: U256, to: H160, data: Vec<u8>) -> Result<H256, Error> {
        let tx_params = self.get_tx_params(value)?;

        let res: Result<(EvmResult<H256>,), _> = ic::call(
            self.get_evm_canister_id(),
            "call_message",
            (tx_params, to, hex::encode(data)),
        )
        .await;
        self.process_call_result(res.map(|val| val.0))
    }

    async fn create_contract(&mut self, value: U256, code: Vec<u8>) -> Result<H256, Error> {
        let tx_params = self.get_tx_params(value)?;

        let res: Result<(EvmResult<H256>,), _> = ic::call(
            self.get_evm_canister_id(),
            "create_contract",
            (tx_params, hex::encode(code)),
        )
        .await;

        self.process_call_result(res.map(|val| val.0))
    }

    async fn get_balance(&self, address: H160) -> Result<U256, Error> {
        let res: Result<(BasicAccount,), _> =
            ic::call(self.get_evm_canister_id(), "account_basic", (address,)).await;

        self.process_call(res.map(|val| val.0))
            .map(|acc| acc.balance)
    }

    async fn get_transaction_by_hash(&self, tx_hash: H256) -> Result<Option<Transaction>, Error> {
        let res: Result<(Option<Transaction>,), _> = ic::call(
            self.get_evm_canister_id(),
            "eth_get_transaction_by_hash",
            (tx_hash,),
        )
        .await;

        self.process_call(res.map(|val| val.0))
    }

    async fn get_transaction_receipt_by_hash(
        &self,
        tx_hash: H256,
    ) -> Result<Option<TransactionReceipt>, Error> {
        let res: Result<(Option<TransactionReceipt>,), _> = ic::call(
            self.get_evm_canister_id(),
            "eth_get_transaction_receipt",
            (tx_hash,),
        )
        .await;

        self.process_call(res.map(|val| val.0))
    }

    async fn mint_evm_tokens(&mut self, to: H160, amount: U256) -> Result<U256, Error> {
        let res: Result<(EvmResult<U256>,), _> =
            ic::call(self.get_evm_canister_id(), "mint_evm_tokens", (to, amount)).await;

        self.process_call_result(res.map(|val| val.0))
    }

    async fn register_ic_agent(&mut self, transaction: Transaction) -> Result<(), Error> {
        let res: Result<(EvmResult<()>,), _> = ic::call(
            self.get_evm_canister_id(),
            "register_ic_agent",
            (transaction,),
        )
        .await;

        self.process_call_result(res.map(|val| val.0))
    }

    async fn verify_registration(&mut self, signing_key: Vec<u8>) -> Result<(), Error> {
        let res: Result<(EvmResult<()>,), _> = ic::call(
            self.get_evm_canister_id(),
            "verify_registration",
            (signing_key,),
        )
        .await;

        self.process_call_result(res.map(|val| val.0))
    }

    async fn is_address_registered(&self, address: H160) -> Result<bool, Error> {
        let res: Result<(bool,), _> = ic::call(
            self.get_evm_canister_id(),
            "is_address_registered",
            (address,),
        )
        .await;

        self.process_call(res.map(|val| val.0))
    }
}

thread_local! {
    static NONCE_CELL: RefCell<StableCell<U256>> = {
        RefCell::new(StableCell::new(NONCE_MEMORY_ID, U256::one())
            .expect("stable memory nonce initialization failed"))
    };
}
