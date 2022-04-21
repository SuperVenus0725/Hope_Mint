use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128,Decimal};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, ReadonlySingleton, Singleton,
};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG_KEY: &[u8] = b"config";
pub const CONFIG_ADDRESS: &[u8] = b"token_address";
pub const CONFIG_NFT_ADDRESS: &[u8] = b"nft_address";
pub const CONFIG_USERS: &[u8] = b"User";
pub const CONFIG_USER_INFO: &[u8] = b"UserInfo";
pub const CONFIG_COUNT: &[u8] = b"TokenCount";
pub const CONFIG_MAXIMUM: &[u8]=b"NFTMaximum";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_nft:Uint128,
    pub owner:String
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalty{
    pub address: String,
    pub royalty_rate:Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenCount{
    pub owned_nft_number:Uint128,
    pub total_nft:Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub address :String,
    pub nft: Vec<String>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata{
    // Identifies the asset to which this NFT represents
    pub name: Option<String>,
    // Describes the asset to which this NFT represents (may be empty)
    pub description: Option<String>,
    // An external URI
    pub external_link: Option<String>,
    // royalties
    pub royalties: Option<Vec<Royalty>>,
    // initial ask price
    pub init_price: Option<Uint128>,
}


pub const USERS: Map<&str, UserInfo> = Map::new("User");

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn store_maximum_nft(storage: &mut dyn Storage,maximum:&Uint128) -> StdResult<()> {
    Singleton::new(storage, CONFIG_MAXIMUM).save(maximum)
}

pub fn read_maximum_nft(storage: &dyn Storage) -> StdResult<Uint128> {
    ReadonlySingleton::new(storage, CONFIG_MAXIMUM).load()
}

pub fn store_token_address(storage: &mut dyn Storage, token_address: &Addr) -> StdResult<()> {
    Singleton::new(storage, CONFIG_ADDRESS).save(token_address)
}

pub fn read_token_address(storage: &dyn Storage) -> StdResult<Addr> {
    ReadonlySingleton::new(storage, CONFIG_ADDRESS).load()
}

pub fn store_nft_address(storage: &mut dyn Storage, nft_address: &Addr) -> StdResult<()> {
    Singleton::new(storage, CONFIG_NFT_ADDRESS).save(nft_address)
}

pub fn read_nft_address(storage: &dyn Storage) -> StdResult<Addr> {
    ReadonlySingleton::new(storage, CONFIG_NFT_ADDRESS).load()
}

pub fn store_users(storage: &mut dyn Storage, user: &Addr, user_info: UserInfo) -> StdResult<()> {
    bucket(storage, CONFIG_USERS).save(user.as_bytes(), &user_info)
}

pub fn read_user_info(storage: &dyn Storage, user: &Addr) -> Option<UserInfo> {
    match bucket_read(storage, CONFIG_USERS).load(user.as_bytes()) {
        Ok(v) => Some(v),
        _ => None,
    }
}

pub fn read_users(storage: &dyn Storage) -> StdResult<Vec<String>> {
    USERS.keys(storage, None, None, Order::Ascending).collect()
}

pub fn store_token_count(storage: &mut dyn Storage, user: &Addr, token_count: Uint128) -> StdResult<()> {
    bucket(storage, CONFIG_COUNT).save(user.as_bytes(), &token_count)
}

pub fn read_token_count(storage: &dyn Storage, user: &Addr) -> Option<Uint128> {
    match bucket_read(storage, CONFIG_COUNT).load(user.as_bytes()) {
        Ok(v) => Some(v),
        _ => None,
    }
}