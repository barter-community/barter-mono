use crate::event::{DataKind, MarketEvent};
use barter_integration::model::{
    instrument::{kind::InstrumentKind, Instrument},
    Exchange,
};
use dotenv::dotenv;
use ethers::{
    contract::{Contract, EthEvent, abigen},
    core::types::{Address, Filter, H160},
    providers::{Provider, StreamExt, Ws},
    core::utils::keccak256,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info};
use reqwest;
use std::collections::HashMap;
use std::error::Error;


abigen!(
    IERC20,
    r#"[
    event Transfer(address indexed from, address indexed to, uint256 value)
    event Approval(address indexed owner, address indexed spender, uint256 value)
]"#,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub outputs: Vec<Output>,
    pub encodedOrder: String,
    pub signature: String,
    pub input: Input,
    pub settledAmounts: Vec<SettledAmount>,
    pub orderStatus: String,
    pub txHash: String,
    pub createdAt: u64,
    pub chainId: u64,
    pub orderHash: String,
    #[serde(rename = "type")]
    pub order_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Output {
    recipient: String,
    startAmount: String,
    endAmount: String,
    token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Input {
    endAmount: String,
    token: String,
    startAmount: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SettledAmount {
    tokenOut: String,
    amountIn: String,
    amountOut: String,
    tokenIn: String,
}
struct Response {
  orders: Vec<Order>,
}

pub struct UniswapX {
  client: reqwest::Client,
  url: String,
}



pub enum DexError {
  Serde(serde_json::Error),
  Reqwest(reqwest::Error),
  Error(String),
}

impl UniswapX {
  pub async fn new() -> Self {
    UniswapX {
      client: reqwest::Client::new(),
      url: "https://api.uniswap.org/v2/orders".to_owned()
    }
  }

  fn deserialize_orders(json_str: &str) -> Result<Vec<Order>, serde_json::Error> {
    // Define a helper struct to match the JSON structure
    #[derive(Deserialize)]
    struct Response {
        orders: Vec<Order>,
    }

    let data: Response = serde_json::from_str(json_str)?;
    Ok(data.orders)
  }

  pub async fn getOpenOrders(&self, chainId: u8) -> Result<Vec<Order>, DexError> {
    let url = format!("{}?chainId={}&orderStatus=open", self.url, chainId);
    let response = self.client.get(&url).send().await?;


    if response.status().is_success() {
        let body: String = response.text().await?;

        // Deserialize the JSON into the defined struct
        let orders = UniswapX::deserialize_orders(&body).unwrap();

        Ok(orders)
    } else {
        Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
    }
  }

  pub async fn getOrderHash(&self, hash: String) -> Result<Order, DexError> {
    let url = format!("{}?orderHash={}", self.url, hash);
    let response = self.client.get(&url).send().await?;

    if response.status().is_success() {
      let body: String = response.text().await?;

      // Deserialize the JSON into the defined struct
      let orders = UniswapX::deserialize_orders(&body).unwrap();
      
      if (orders.len() == 1) {
        Ok(orders[0].clone())
      } else {
        Err(DexError::Error("Order not found".to_owned()))
      }
    } else {
      Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
    }
  }

  pub async fn fetch(&self, url: &str) -> Result<String, reqwest::Error> {
    let response = self.client.get(url).send().await?;

    if response.status().is_success() {
        let body: String = response.text().await?;
        Ok(body)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
  }
}

impl From<serde_json::Error> for DexError {
  fn from(err: serde_json::Error) -> DexError {
      DexError::Serde(err)
  }
}

impl From<reqwest::Error> for DexError {
  fn from(err: reqwest::Error) -> DexError {
      DexError::Reqwest(err)
  }
}

// Implement dummy Serializiation (not used when running code)
impl<'de> Deserialize<'de> for TransferFilter {
    fn deserialize<D>(_deserializer: D) -> Result<TransferFilter, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserialize is not implemented for TransferFilter")
    }
}

impl Serialize for TransferFilter {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!("Serialize is not implemented for TransferFilter")
    }
}
