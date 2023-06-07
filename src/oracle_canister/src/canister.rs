use std::time::Duration;

use candid::{CandidType, Deserialize};
use ic_canister::{generate_idl, init, post_upgrade, query, update, Canister, Idl, PreUpdate};
use ic_exports::ic_cdk;
use ic_exports::ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_exports::ic_cdk_timers::set_timer_interval;
use ic_exports::ic_kit::ic;
use ic_exports::Principal;

use crate::error::{Error, Result};
use crate::evm_canister::contract::ContractService;
use crate::evm_canister::did::{Transaction, H160, H256, U256};
use crate::state::http::{http, HttpRequest as ServeRequest, HttpResponse as ServeHttpResponse};
use crate::state::{PairKey, PairPrice, Settings, State};
use crate::timer::{sync_coinbase_price, sync_coingecko_price, transform};

/// A canister to transfer funds between IC token canisters and EVM canister contracts.
#[derive(Canister)]
pub struct OracleCanister {
    #[id]
    id: Principal,
    state: State,
}

impl PreUpdate for OracleCanister {}

impl OracleCanister {
    /// Initialize the canister with given data.
    #[init]
    pub fn init(&mut self, init_data: InitData) {
        let settings = Settings {
            owner: init_data.owner,
            evmc_principal: init_data.evmc_principal,
        };

        self.state.reset(settings);

        #[cfg(target_arch = "wasm32")]
        crate::timer::wasm32::init_timer(self.state.pair_price);
    }

    /// Returns principal of canister owner.
    #[query]
    pub fn get_owner(&self) -> Principal {
        self.state.config.get_owner()
    }

    /// Sets a new principal for canister owner.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    #[update]
    pub fn set_owner(&mut self, owner: Principal) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.config.set_owner(owner);
        Ok(())
    }

    /// Returns principal of EVM canister with which the oracle canister works.
    #[query]
    pub fn get_evmc_principal(&self) -> Principal {
        self.state.config.get_evmc_principal()
    }

    /// Sets principal of EVM canister with which the oracle canister works.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    #[update]
    pub fn set_evmc_principal(&mut self, evmc: Principal) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.config.set_evmc_principal(evmc);
        Ok(())
    }

    /// Returns the all types of price pairs
    #[query]
    pub fn get_pairs(&self) -> Vec<String> {
        self.state
            .pair_price
            .get_pairs()
            .iter()
            .map(|p| p.0.clone())
            .collect()
    }

    /// Returns the latest (timestamp, price) of given pair
    #[query]
    pub fn get_latest_price(&self, pair: String) -> Result<(u64, u64)> {
        let pair_key = PairKey(pair);
        if !self.state.pair_price.is_exist(&pair_key) {
            return Err(Error::PairNotExist);
        }
        self.state
            .pair_price
            .get_latest_price(&pair_key)
            .ok_or(Error::Internal(
                "latest price for this pair doesn't exist.".to_string(),
            ))
    }

    /// Return the latest n records of a price pair, or fewer if the price's amount fewer
    #[query]
    pub fn get_prices(&self, pair: String, n: usize) -> Vec<(u64, u64)> {
        self.state.pair_price.get_prices(&PairKey(pair), n)
    }

    /// Adds a new pair to the oracle canister.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    ///
    /// If `pair` is used already, `Error::PairExist` will be returned.
    #[update]
    pub fn add_pair(&mut self, pair: String) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.pair_price.add_pair(PairKey(pair))
    }

    /// Remove the given pair from the oracle canister.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    ///
    /// If there is no pair for `pair`, `Error::PairNotExist` will be returned.
    #[update]
    pub fn remove_pair(&mut self, pair: String) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.pair_price.del_pair(PairKey(pair))
    }

    /// Manually trigger http outcalls to update the price of the specified pair in this canister
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    ///
    /// If there is no pair for `pair`, `Error::PairNotExist` will be returned.
    #[update]
    pub async fn update_price(&mut self, pairs: Vec<String>, api: ApiType) -> Result<()> {
        self.check_owner(ic::caller())?;

        let mut pair_keys = Vec::new();
        for pair_key in pairs.into_iter().map(PairKey) {
            if !self.state.pair_price.is_exist(&pair_key) {
                return Err(Error::PairNotExist);
            }
            pair_keys.push(pair_key);
        }

        match api {
            ApiType::Coinbase => {
                sync_coinbase_price(pair_keys[0].clone(), &mut self.state.pair_price).await
            }
            ApiType::Coingecko => sync_coingecko_price(pair_keys, &mut self.state.pair_price).await,
        }
    }

    #[update]
    pub async fn start_feed_price_timer(&self) {
        set_timer_interval(Duration::from_secs(300), move || {
            let pair_price = PairPrice::default();
            let pair_keys = pair_price.get_pairs();
            let pairs = pair_keys
                .clone()
                .into_iter()
                .map(|p| p.0)
                .collect::<Vec<String>>();
            let (timestamps, prices) = pair_keys
                .iter()
                .map(|p| pair_price.get_latest_price(p).expect("no latest price"))
                .map(|(t, p)| (t.into(), p.into()))
                .unzip();

            ic_cdk::spawn(async move {
                let contract = ContractService::default();
                let res = contract.update_answers(pairs, timestamps, prices).await;
                ic::print(format!("res: {res:?}"));
            });
        });
    }

    /// Runs the procedure of registering this canister's account in evmc.
    #[update]
    pub async fn register_self_in_evmc(
        &mut self,
        transaction: Transaction,
        signing_key: Vec<u8>,
    ) -> Result<()> {
        self.check_owner(ic::caller())?;

        self.state
            .self_account
            .register_account(transaction, signing_key)
            .await
    }

    #[update]
    pub fn reset_self_account(&mut self) -> Result<()> {
        self.check_owner(ic::caller())?;

        self.state.self_account.reset();
        Ok(())
    }

    /// Returns this canister's account in evmc if registered
    #[query]
    pub fn get_self_address_in_evmc(&self) -> Result<H160> {
        self.state.self_account.get_account()
    }

    /// deploy the AggregatorSingle contract to evmc, and stored the tx hash.
    #[update]
    pub async fn deploy_aggregator_contract(&mut self) -> Result<H256> {
        self.check_owner(ic::caller())?;

        self.state.contract.init_contract().await
    }

    // Make sure the deployment is successful and get the contract address from the transaction receipt
    #[update]
    pub async fn confirm_aggregator_contract(&mut self) -> Result<H160> {
        self.check_owner(ic::caller())?;

        self.state.contract.confirm_contract_address().await
    }

    /// Returns the aggregator contract address if deployed
    #[query]
    pub fn get_aggregator_contract_address(&self) -> Result<H160> {
        self.state.contract.get_contract()
    }

    /// Call the Aggregator contract's `addPair` in evmc to increase the currency price pairs supported by the aggregator
    #[update]
    pub async fn add_pair_in_aggregator(
        &self,
        pair: String,
        decimal: U256,
        description: String,
        version: U256,
    ) -> Result<H256> {
        self.check_owner(ic::caller())?;

        self.state
            .contract
            .add_pair(pair, decimal, description, version)
            .await
    }

    /// Call the Aggregator contract's `updateAnswers` in evmc to update the supported currency price pairs.
    #[update]
    pub async fn update_answers(
        &self,
        pairs: Vec<String>,
        timestamps: Vec<U256>,
        prices: Vec<U256>,
    ) -> Result<H256> {
        self.check_owner(ic::caller())?;

        self.state
            .contract
            .update_answers(pairs, timestamps, prices)
            .await
    }

    #[query]
    fn http_request(&self, req: ServeRequest) -> ServeHttpResponse {
        let now = ic::time();
        http(req, now, &self.state.pair_price)
    }

    fn check_owner(&self, principal: Principal) -> Result<()> {
        let owner = self.state.config.get_owner();
        if owner == principal || owner == Principal::anonymous() {
            return Ok(());
        }
        Err(Error::NotAuthorized)
    }

    /// Requirements for Http outcalls, used to ignore small differences in the data obtained
    /// by different nodes of the IC subnet to reach a consensus, more info:
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/http_requests/http_requests-how-it-works#transformation-function
    #[query]
    fn transform(&self, raw: TransformArgs) -> HttpResponse {
        transform(raw)
    }

    #[post_upgrade]
    fn post_upgrade(&self) {
        #[cfg(target_arch = "wasm32")]
        crate::timer::wasm32::init_timer(self.state.pair_price);
    }

    /// Returns candid IDL.
    /// This should be the last fn to see previous endpoints in macro.
    pub fn idl() -> Idl {
        generate_idl!()
    }
}

/// Oracle canister initialization data.
#[derive(Debug, Deserialize, CandidType, Clone, Copy)]
pub struct InitData {
    /// Principal of canister's owner.
    pub owner: Principal,

    /// Principal of EVM canister, in which Oracle canister will mint/burn tokens.
    pub evmc_principal: Principal,
}

#[derive(Debug, Deserialize, CandidType, Clone, Copy)]
pub enum ApiType {
    Coinbase,
    Coingecko,
}
