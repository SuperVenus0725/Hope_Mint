use crate::state::Royalty;
use cosmwasm_std::{ Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    SetTokenAddress { address: String },
    SetNftAddress { address: String },
    SetMaximumNft {amount: Uint128},
    BuyToken { amount:i32 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetTokenAddress {},
    GetBalance { address: String },
    GetContractInfo {},
    GetTokenInfo { address: String },
    GetTokenCount{address:String},
    GetNftAddress {},
    GetAllUsers {},
    GetUserInfo { address: String },
    GetMaximumNft{},
    GetStateInfo{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct HopeMintMsg {
    // Identifies the asset to which this NFT represents
    pub name: Option<String>,
    // A URI pointing to an image representing the asset
    pub image_uri: Option<String>,
    // An external URI
    pub external_link: Option<String>,
    // Describes the asset to which this NFT represents (may be empty)
    pub description: Option<String>,
    // royalties
    pub royalties: Option<Vec<Royalty>>,
    // initial ask price
    pub init_price: Option<Uint128>,
    // nft address of specified collection
    pub nft_addr: Option<String>,
}
