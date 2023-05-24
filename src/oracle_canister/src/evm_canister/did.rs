use std::borrow::Cow;
use std::fmt;
use std::ops::Add;
use std::str::FromStr;

use candid::{CandidType, Decode, Encode};
use derive_more::Display;
use ic_stable_structures::{BoundedStorable, Storable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(
    Debug, Default, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize, Display, Hash,
)]
#[serde(transparent)]
pub struct Hash<T>(pub T);

///Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
pub type H160 = Hash<ethereum_types::H160>;
///Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
pub type H256 = Hash<ethereum_types::H256>;

#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
#[serde(transparent)]
pub struct U64(pub ethereum_types::U64);

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct U256(pub ethereum_types::U256);

pub fn encode(item: &impl CandidType) -> Vec<u8> {
    Encode!(item).expect("failed to encode item to candid")
}

pub fn decode<'a, T: CandidType + Deserialize<'a>>(bytes: &'a [u8]) -> T {
    Decode!(bytes, T).expect("failed to decode item from candid")
}

pub fn from_hex_str<const SIZE: usize>(mut s: &str) -> Result<[u8; SIZE], hex::FromHexError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        s = &s[2..];
    }

    let mut result = [0u8; SIZE];
    hex::decode_to_slice(s, &mut result).and(Ok(result))
}

impl H160 {
    pub fn new(value: ethereum_types::H160) -> Self {
        Self(value)
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self(ethereum_types::H160::from_slice(slice))
    }

    pub fn from_hex_str(s: &str) -> Result<Self, hex::FromHexError> {
        Ok(Self(ethereum_types::H160::from(from_hex_str::<20>(s)?)))
    }

    pub fn to_hex_str(&self) -> String {
        format!("0x{self:x}")
    }

    pub const fn zero() -> Self {
        Self(ethereum_types::H160::zero())
    }
}

impl H256 {
    pub fn new(value: ethereum_types::H256) -> Self {
        Self(value)
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self(ethereum_types::H256::from_slice(slice))
    }

    pub fn from_hex_str(s: &str) -> Result<Self, hex::FromHexError> {
        Ok(Self(ethereum_types::H256::from(from_hex_str::<32>(s)?)))
    }

    pub fn to_hex_str(&self) -> String {
        format!("0x{self:x}")
    }

    pub const fn zero() -> Self {
        Self(ethereum_types::H256::zero())
    }
}

impl U256 {
    pub const BYTE_SIZE: usize = 32;

    pub fn new(value: ethereum_types::U256) -> Self {
        Self(value)
    }

    pub fn max_value() -> Self {
        Self(ethereum_types::U256::max_value())
    }

    pub fn from_hex_str(mut s: &str) -> Result<Self, String> {
        if s.starts_with("0x") || s.starts_with("0X") {
            s = &s[2..]
        }
        ethereum_types::U256::from_str(s)
            .map_err(|e| e.to_string())
            .map(Into::into)
    }

    pub fn to_hex_str(&self) -> String {
        format!("0x{self:x}")
    }

    pub const fn zero() -> Self {
        Self(ethereum_types::U256::zero())
    }

    pub const fn one() -> Self {
        Self(ethereum_types::U256::one())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn to_big_endian(&self) -> Vec<u8> {
        let mut buffer = vec![0; 32];
        self.0.to_big_endian(&mut buffer);
        buffer
    }

    pub fn from_big_endian(slice: &[u8]) -> Self {
        Self(ethereum_types::U256::from_big_endian(slice))
    }

    pub fn to_little_endian(&self) -> Vec<u8> {
        let mut buffer = vec![0; 32];
        self.0.to_little_endian(&mut buffer);
        buffer
    }

    pub fn from_little_endian(slice: &[u8]) -> Self {
        Self(ethereum_types::U256::from_little_endian(slice))
    }

    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl U64 {
    pub const BYTE_SIZE: usize = 8;

    pub fn new(value: ethereum_types::U64) -> Self {
        Self(value)
    }

    pub fn max_value() -> Self {
        Self(ethereum_types::U64::max_value())
    }

    pub fn from_hex_str(mut s: &str) -> Result<Self, String> {
        if s.starts_with("0x") || s.starts_with("0X") {
            s = &s[2..]
        }
        ethereum_types::U64::from_str(s)
            .map_err(|e| e.to_string())
            .map(Into::into)
    }

    pub fn to_hex_str(&self) -> String {
        format!("0x{self:x}")
    }

    pub const fn zero() -> Self {
        Self(ethereum_types::U64::zero())
    }

    pub const fn one() -> Self {
        Self(ethereum_types::U64::one())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn to_big_endian(&self) -> Vec<u8> {
        let mut buffer = vec![0; 8];
        self.0.to_big_endian(&mut buffer);
        buffer
    }

    pub fn from_big_endian(slice: &[u8]) -> Self {
        Self(ethereum_types::U64::from_big_endian(slice))
    }

    pub fn to_little_endian(&self) -> Vec<u8> {
        let mut buffer = vec![0; 8];
        self.0.to_little_endian(&mut buffer);
        buffer
    }

    pub fn from_little_endian(slice: &[u8]) -> Self {
        Self(ethereum_types::U64::from_little_endian(slice))
    }
}

impl Storable for H160 {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0.as_ref().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(ethereum_types::H160::from_slice(bytes.as_ref()))
    }
}

impl Storable for H256 {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0.as_ref().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(ethereum_types::H256::from_slice(bytes.as_ref()))
    }
}

impl Storable for U256 {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        self.to_big_endian().into()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self::from_big_endian(bytes.as_ref())
    }
}

impl BoundedStorable for H160 {
    const MAX_SIZE: u32 = 20;
    const IS_FIXED_SIZE: bool = true;
}

impl BoundedStorable for H256 {
    const MAX_SIZE: u32 = 32;
    const IS_FIXED_SIZE: bool = true;
}

impl BoundedStorable for U256 {
    const MAX_SIZE: u32 = 32;
    const IS_FIXED_SIZE: bool = true;
}

impl CandidType for H160 {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.to_hex_str())
    }
}

impl CandidType for H256 {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.to_hex_str())
    }
}

impl CandidType for U64 {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.to_hex_str())
    }
}

impl CandidType for U256 {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.to_hex_str())
    }
}

impl Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.0 + rhs.0)
    }
}

impl rlp::Encodable for H160 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.0.rlp_append(s);
    }
}

impl rlp::Decodable for H160 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        ethereum_types::H160::decode(rlp).map(Into::into)
    }
}

impl rlp::Encodable for H256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.0.rlp_append(s);
    }
}

impl rlp::Decodable for H256 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        ethereum_types::H256::decode(rlp).map(Into::into)
    }
}

impl rlp::Encodable for U256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.0.rlp_append(s);
    }
}

impl rlp::Decodable for U256 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        ethereum_types::U256::decode(rlp).map(Into::into)
    }
}

impl From<ethereum_types::U64> for U64 {
    fn from(v: ethereum_types::U64) -> Self {
        Self(v)
    }
}

impl From<ethereum_types::U256> for U256 {
    fn from(v: ethereum_types::U256) -> Self {
        Self(v)
    }
}

impl From<U256> for ethereum_types::U256 {
    fn from(value: U256) -> Self {
        value.0
    }
}

impl From<H160> for ethereum_types::H160 {
    fn from(value: H160) -> Self {
        value.0
    }
}

impl From<H256> for ethereum_types::H256 {
    fn from(value: H256) -> Self {
        value.0
    }
}

impl From<ethereum_types::H160> for H160 {
    fn from(value: ethereum_types::H160) -> Self {
        Hash(value)
    }
}

impl From<ethereum_types::H256> for H256 {
    fn from(value: ethereum_types::H256) -> Self {
        Hash(value)
    }
}

impl fmt::LowerHex for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for U64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for U64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize, Default)]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    #[serde(default, rename = "blockHash")]
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    #[serde(default, rename = "blockNumber")]
    pub block_number: Option<U64>,

    /// Transaction Index. None when pending.
    #[serde(default, rename = "transactionIndex")]
    pub transaction_index: Option<U64>,

    /// Sender
    #[serde(default)]
    pub from: H160,

    /// Recipient (None when contract creation)
    #[serde(default)]
    pub to: Option<H160>,

    /// Transferred value
    pub value: U256,

    /// Gas Price, null for Type 2 transactions
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,

    /// Gas amount
    pub gas: U256,

    /// Input data
    pub input: Bytes,

    /// ECDSA recovery id
    pub v: U64,

    /// ECDSA signature r
    pub r: U256,

    /// ECDSA signature s
    pub s: U256,

    // EIP2718
    /// Transaction type, Some(2) for EIP-1559 transaction,
    /// Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,

    // EIP2930
    #[serde(
        rename = "accessList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub access_list: Option<AccessList>,

    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    #[serde(
        rename = "maxPriorityFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_priority_fee_per_gas: Option<U256>,

    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    #[serde(
        rename = "maxFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_fee_per_gas: Option<U256>,

    #[serde(rename = "chainId", default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<U256>,
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize, Default)]
pub struct AccessListItem {
    pub address: H160,
    #[serde(default, rename = "storageKeys")]
    pub storage_keys: Vec<H256>,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq, Debug, CandidType)]
pub struct AccessList(pub Vec<AccessListItem>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Bytes(pub bytes::Bytes);

impl Bytes {
    pub fn from_hex_str(mut s: &str) -> Result<Self, hex::FromHexError> {
        if s.starts_with("0x") || s.starts_with("0X") {
            s = &s[2..]
        }
        let bytes = hex::decode(s)?;
        Ok(Self(bytes::Bytes::from(bytes)))
    }

    pub fn to_hex_str(&self) -> String {
        format!("0x{self:x}")
    }
}

impl fmt::LowerHex for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl CandidType for Bytes {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.to_hex_str())
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex_str())
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Bytes::from_hex_str(&value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum BlockNumber {
    Latest,
    Earliest,
    Pending,
    Number(U64),
}

impl Serialize for BlockNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockNumber::Latest => serializer.serialize_str("latest"),
            BlockNumber::Earliest => serializer.serialize_str("earliest"),
            BlockNumber::Pending => serializer.serialize_str("pending"),
            BlockNumber::Number(ref n) => serializer.serialize_str(&n.to_hex_str()),
        }
    }
}

impl<'de> Deserialize<'de> for BlockNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        Ok(match s.as_str() {
            "latest" => Self::Latest,
            "earliest" => Self::Earliest,
            "pending" => Self::Pending,
            n => BlockNumber::Number(U64::from_hex_str(n).map_err(serde::de::Error::custom)?),
        })
    }
}

impl CandidType for BlockNumber {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Text
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        match *self {
            BlockNumber::Latest => serializer.serialize_text("latest"),
            BlockNumber::Earliest => serializer.serialize_text("earliest"),
            BlockNumber::Pending => serializer.serialize_text("pending"),
            BlockNumber::Number(ref n) => serializer.serialize_text(&format!("0x{n:x}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, CandidType)]
pub struct TransactionParams {
    pub from: H160,
    pub value: U256,
    pub gas_limit: u64,
    pub gas_price: Option<U256>,
    pub nonce: U256,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, CandidType)]
pub struct BasicAccount {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: U256,
}
