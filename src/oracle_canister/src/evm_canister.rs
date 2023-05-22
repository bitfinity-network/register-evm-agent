use std::cell::RefCell;

use async_trait::async_trait;
use did::error::{EvmError, TransactionPoolError};
#[cfg(target_arch = "wasm32")]
use did::BasicAccount;
use did::{BlockNumber, Transaction, TransactionParams, TransactionReceipt, H160, H256, U256};
use evmc::ic::api::EvmCanister as Evmc;
use ic_canister::{canister_call, Canister};
use ic_exports::ic_kit::RejectionCode;
use ic_stable_structures::StableCell;
use mockall::automock;

use crate::constant::{DEFAULT_GAS_LIMIT, NONCE_MEMORY_ID};
use crate::error::Error;
use crate::state::State;

/// Interface for calling EVMC methods
#[automock]
#[async_trait(?Send)]
pub trait EvmCanister: Send {
    async fn transact(&mut self, value: U256, to: H160, data: Vec<u8>) -> Result<H256, Error>;

    async fn send_raw_transaction(&mut self, tx: Transaction) -> Result<H256, Error>;

    async fn create_contract(&mut self, value: U256, code: Vec<u8>) -> Result<H256, Error>;

    async fn get_contract_code(&self, address: H160) -> Result<Vec<u8>, Error>;

    async fn get_balance(&self, address: H160) -> Result<U256, Error>;

    async fn get_transaction_by_hash(&self, tx_hash: H256) -> Result<Option<Transaction>, Error>;

    async fn get_transaction_receipt_by_hash(
        &self,
        tx_hash: H256,
    ) -> Result<Option<TransactionReceipt>, Error>;

    async fn deposit(&mut self, to: H160, amount: U256) -> Result<U256, Error>;

    async fn register_ic_agent(&mut self, transaction: Transaction) -> Result<(), Error>;

    async fn verify_registration(&mut self, signing_key: Vec<u8>) -> Result<(), Error>;
}

#[derive(Default)]
pub struct EvmCanisterImpl {}

impl EvmCanisterImpl {
    fn get_evmc_canister(&self) -> Evmc {
        Evmc::from_principal(State::default().config.get_evmc_principal())
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
        let state = State::default();
        Ok(TransactionParams {
            from: state.account.get_account()?,
            value,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_price: None,
            nonce: self.get_nonce(),
        })
    }

    pub fn reset(&self) {
        NONCE_CELL.with(|nonce| {
            nonce
                .borrow_mut()
                .set(U256::one())
                .expect("failed to update nonce");
        });
    }
}

type EvmResult<T> = Result<T, EvmError>;

#[async_trait(?Send)]
impl EvmCanister for EvmCanisterImpl {
    async fn transact(&mut self, value: U256, to: H160, data: Vec<u8>) -> Result<H256, Error> {
        let mut evmc = self.get_evmc_canister();
        let tx_params = self.get_tx_params(value)?;

        self.process_call_result(
            canister_call!(
                evmc.call_message(tx_params, to, hex::encode(data)),
                EvmResult<H256>
            )
            .await,
        )
    }

    async fn send_raw_transaction(&mut self, tx: Transaction) -> Result<H256, Error> {
        let mut evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(evmc.send_raw_transaction(tx), EvmResult<H256>).await,
        )
    }

    async fn create_contract(&mut self, value: U256, code: Vec<u8>) -> Result<H256, Error> {
        let mut evmc = self.get_evmc_canister();
        let tx_params = self.get_tx_params(value)?;
        self.process_call_result(
            canister_call!(
                evmc.create_contract(tx_params, hex::encode(code)),
                EvmResult<H256>
            )
            .await,
        )
    }

    async fn get_contract_code(&self, address: H160) -> Result<Vec<u8>, Error> {
        let evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(
                evmc.eth_get_code(address, BlockNumber::Latest),
                EvmResult<String>
            )
            .await,
        )
        .and_then(|code| {
            hex::decode(code)
                .map_err(|_| Error::Internal("failed to decode contract code".to_string()))
        })
    }

    async fn get_balance(&self, address: H160) -> Result<U256, Error> {
        let evmc = self.get_evmc_canister();
        self.process_call(canister_call!(evmc.account_basic(address), BasicAccount).await)
            .map(|acc| acc.balance)
    }

    async fn get_transaction_by_hash(&self, tx_hash: H256) -> Result<Option<Transaction>, Error> {
        let evmc = self.get_evmc_canister();
        self.process_call(
            canister_call!(
                evmc.eth_get_transaction_by_hash(tx_hash),
                Option<Transaction>
            )
            .await,
        )
    }

    async fn get_transaction_receipt_by_hash(
        &self,
        tx_hash: H256,
    ) -> Result<Option<TransactionReceipt>, Error> {
        let evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(
                evmc.eth_get_transaction_receipt(tx_hash),
                EvmResult<Option<TransactionReceipt>>
            )
            .await,
        )
    }

    async fn deposit(&mut self, to: H160, amount: U256) -> Result<U256, Error> {
        let mut evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(evmc.deposit_tokens(to, amount), EvmResult<U256>).await,
        )
    }

    async fn register_ic_agent(&mut self, transaction: Transaction) -> Result<(), Error> {
        let mut evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(evmc.register_ic_agent(transaction), EvmResult<()>).await,
        )
    }

    async fn verify_registration(&mut self, signing_key: Vec<u8>) -> Result<(), Error> {
        let mut evmc = self.get_evmc_canister();
        self.process_call_result(
            canister_call!(evmc.verify_registration(signing_key), EvmResult<()>).await,
        )
    }
}

thread_local! {
    static NONCE_CELL: RefCell<StableCell<U256>> = {
        RefCell::new(StableCell::new(NONCE_MEMORY_ID, U256::one())
            .expect("stable memory nonce initialization failed"))
    };
}
