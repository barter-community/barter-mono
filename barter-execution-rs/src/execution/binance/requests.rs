use barter_integration::{model::instrument::symbol::Symbol, protocol::http::rest::ApiRequest};
use serde::Deserialize;

use crate::model::balance::{Balance, SymbolBalance};

pub const SPOT_BALANCES_REQUEST: ApiRequest<BalancesResponse, ()> = ApiRequest::new(
    "/sapi/v3/asset/getUserAsset",
    reqwest::Method::POST,
    "fetch_balances",
);

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

pub const FUT_BALANCES_REQUEST: ApiRequest<FutBalancesResponse, ()> = ApiRequest::new(
    "/fapi/v2/balance",
    reqwest::Method::GET,
    "fetch_fut_balances",
);

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

// FUT ORDER

pub const FUT_ORDER_REQUEST: ApiRequest<FutOrderResponse, FutOrderResponse> =
    ApiRequest::new("/fapi/v1/order", reqwest::Method::POST, "fut_order");

// define order filled type for futures binance order with public fields and conversion to f64
// {
//     "clientOrderId": "testOrder",
//     "cumQty": "0",
//     "cumQuote": "0",
//     "executedQty": "0",
//     "orderId": 22542179,
//     "avgPrice": "0.00000",
//     "origQty": "10",
//     "price": "0",
//     "reduceOnly": false,
//     "side": "BUY",
//     "positionSide": "SHORT",
//     "status": "NEW",
//     "stopPrice": "9300",        // please ignore when order type is TRAILING_STOP_MARKET
//     "closePosition": false,   // if Close-All
//     "symbol": "BTCUSDT",
//     "timeInForce": "GTD",
//     "type": "TRAILING_STOP_MARKET",
//     "origType": "TRAILING_STOP_MARKET",
//     "activatePrice": "9020",    // activation price, only return with TRAILING_STOP_MARKET order
//     "priceRate": "0.3",         // callback rate, only return with TRAILING_STOP_MARKET order
//     "updateTime": 1566818724722,
//     "workingType": "CONTRACT_PRICE",
//     "priceProtect": false,      // if conditional order trigger is protected
//     "priceMatch": "NONE",              //price match mode
//     "selfTradePreventionMode": "NONE", //self trading preventation mode
//     "goodTillDate": 1693207680000      //order pre-set auot cancel time for TIF GTD order
// }
#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
pub struct FutOrderResponse {
    pub clientOrderId: String,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub cumQty: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub cumQuote: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub executedQty: f64,
    pub orderId: u64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub avgPrice: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub origQty: f64,
    #[serde(deserialize_with = "barter_integration::de::de_str")]
    pub price: f64,
    pub reduceOnly: bool,
    pub side: String,
    pub positionSide: String,
    pub status: String,
    pub stopPrice: String,
    pub closePosition: bool,
    pub symbol: Symbol,
    pub timeInForce: String,
    pub r#type: String,
    pub origType: String,
    // pub activatePrice: String,
    // pub priceRate: f64,
    pub updateTime: u64,
    pub workingType: String,
    pub priceProtect: bool,
    pub priceMatch: String,
    pub selfTradePreventionMode: String,
    pub goodTillDate: u64,
}
