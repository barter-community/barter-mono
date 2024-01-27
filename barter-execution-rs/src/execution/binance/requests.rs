use std::marker::PhantomData;

use barter_integration::{
    model::instrument::symbol::Symbol,
    protocol::http::rest::{make_api_req, ApiRequest, SimpleGetRequest},
};
use serde::Deserialize;

use crate::model::balance::{Balance, SymbolBalance};

pub const SPOT_BALANCES_REQUEST: ApiRequest<BalancesResponse, ()> = ApiRequest {
    path: "/sapi/v3/asset/getUserAsset",
    method: reqwest::Method::POST,
    tag_method: "fetch_balances",
    body: None,
    query_params: None,
    response: PhantomData,
};

#[derive(Debug, Deserialize)]
pub struct BalancesResponse(Vec<BinanceBalance>);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BinanceBalance {
    asset: Symbol,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    free: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    freeze: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    withdrawing: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    ipoable: f64,
}
impl From<BinanceBalance> for SymbolBalance {
    fn from(balance: BinanceBalance) -> Self {
        Self {
            symbol: balance.asset,
            balance: Balance {
                total: balance.free,
                available: balance.free - balance.freeze - balance.withdrawing - balance.ipoable,
            },
        }
    }
}

impl From<BalancesResponse> for Vec<SymbolBalance> {
    fn from(vec_t: BalancesResponse) -> Vec<SymbolBalance> {
        vec_t.0.into_iter().map(SymbolBalance::from).collect()
    }
}

// FUTURES BALANCES ***NOTE*** api endpoint is different

// TODO: is this worth it?
pub const FUT_BALANCES_REQUEST: ApiRequest<FutBalancesResponse, ()> =
    make_api_req(SimpleGetRequest {
        path: "/fapi/v2/balance",
        tag_method: "fetch_fut_balances",
        response: PhantomData,
    });

#[derive(Debug, Deserialize)]
pub struct FutBalancesResponse(Vec<FutBalance>);

#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
pub struct FutBalance {
    accountAlias: String, // account alias
    asset: Symbol,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    balance: f64, // wallet balance
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    crossWalletBalance: f64, // crossed wallet balance
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    crossUnPnl: f64, // unrealized profit of crossed positions
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    availableBalance: f64, // available balance
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    maxWithdrawAmount: f64, // maximum amount for transfer out
    marginAvailable: bool, // whether the asset can be used as margin in Multi-Assets mode
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    updateTime: u64,
}

impl From<FutBalance> for SymbolBalance {
    fn from(balance: FutBalance) -> Self {
        Self {
            symbol: balance.asset,
            balance: Balance {
                total: balance.balance,
                available: balance.availableBalance,
            },
        }
    }
}

impl From<FutBalancesResponse> for Vec<SymbolBalance> {
    fn from(vec_t: FutBalancesResponse) -> Vec<SymbolBalance> {
        vec_t.0.into_iter().map(SymbolBalance::from).collect()
    }
}
