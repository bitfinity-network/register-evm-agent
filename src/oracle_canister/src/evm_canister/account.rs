use std::borrow::Cow;
use std::cell::RefCell;

use candid::{CandidType, Deserialize};
use ic_stable_structures::{StableCell, Storable};

use crate::error::{Error, Result};
use crate::evm_canister::did::{decode, encode, Transaction, H160, U256};
use crate::evm_canister::{EvmCanisterImpl, REGISTRATION_FEE};
use crate::state::{ACCOUNT_MEMORY_ID, NONCE_MEMORY_ID};

use super::EvmCanister;

#[derive(Default, Clone)]
pub struct Account {}

impl Account {
    /// Returns this canister's account in evmc if registered
    pub fn get_account(&self) -> Result<H160> {
        ACCOUNT_DATA_CELL.with(|account_data| {
            if let AccountState::Registered(address) = account_data.borrow().get() {
                Ok(address.clone())
            } else {
                Err(Error::Internal("Account no registered yet".to_string()))
            }
        })
    }

    /// Runs the procedure of registering this canister's account in evmc.
    #[allow(dead_code)]
    pub async fn register_account(
        &mut self,
        transaction: Transaction,
        signing_key: Vec<u8>,
    ) -> Result<()> {
        // check if account is alrewady registered or in process
        if ACCOUNT_DATA_CELL.with(|account| {
            if account.borrow().get() == &AccountState::Unregistered {
                account
                    .borrow_mut()
                    .set(AccountState::RegistrationInProgress)
                    .expect("failed to update account state");
                false
            } else {
                true
            }
        }) {
            return Err(Error::Internal("Account already registered".to_string()));
        }

        let mut evm_impl = EvmCanisterImpl::default();

        let address = transaction.from.clone();

        // check if the address is regestry
        match evm_impl.is_address_registered(address.clone()).await {
            Err(err) => {
                self.reset();
                return Err(err);
            }
            Ok(is_registered) => {
                if is_registered {
                    self.reset();
                    return Err(Error::Internal(format!(
                        "{} is already registered",
                        address.clone()
                    )));
                }
            }
        }

        // mint EVM native tokens to from address
        if let Err(err) = evm_impl
            .mint_evm_tokens(address.clone(), REGISTRATION_FEE.into())
            .await
        {
            self.reset();
            return Err(err);
        }

        // register ic agent
        if let Err(err) = evm_impl.register_ic_agent(transaction).await {
            self.reset();
            return Err(err);
        }

        // verify the key
        if let Err(err) = evm_impl.verify_registration(signing_key).await {
            self.reset();
            return Err(err);
        }

        ACCOUNT_DATA_CELL.with(|account| {
            account
                .borrow_mut()
                .set(AccountState::Registered(address))
                .expect("failed to update account state")
        });

        Ok(())
    }

    /// Set the account state as unregistered
    pub fn reset(&mut self) {
        ACCOUNT_DATA_CELL.with(|account| {
            account
                .borrow_mut()
                .set(AccountState::Unregistered)
                .expect("failed to update account state")
        })
    }
}

#[derive(Debug, Default, CandidType, Deserialize, PartialEq, Eq)]
enum AccountState {
    #[default]
    Unregistered,
    RegistrationInProgress,
    Registered(H160),
}

impl Storable for AccountState {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(self).into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(&bytes)
    }
}

thread_local! {
    static ACCOUNT_DATA_CELL: RefCell<StableCell<AccountState>> = {
        RefCell::new(StableCell::new(ACCOUNT_MEMORY_ID, AccountState::default())
            .expect("stable memory account initialization failed"))
    };

    static NONCE_CELL: RefCell<StableCell<U256>> = {
        RefCell::new(StableCell::new(NONCE_MEMORY_ID, U256::one())
            .expect("stable memory nonce initialization failed"))
    };
}
