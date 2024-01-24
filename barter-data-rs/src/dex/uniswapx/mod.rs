// use crate::event::{DataKind, MarketEvent};
// use barter_integration::model::{
//     instrument::{kind::InstrumentKind, Instrument}
// };
use eyre::Result;
use serde::Deserialize;
// use std::sync::Arc;
// use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
// use tracing::{debug, info};
use reqwest;
// use std::collections::HashMap;
// use std::error::Error;
use self::order::{Order, Response};
use super::DexError;

pub mod order;

const UNISWAPX_API: &str = "https://api.uniswap.org/v2/orders";

pub async fn get_open_orders(chainId: u8) -> Result<Vec<Order>, DexError> {
  let url = format!("{}?chainId={}&orderStatus=open", UNISWAPX_API, chainId);
  let response = reqwest::get(&url).await?;


  if response.status().is_success() {
      let body: String = response.text().await?;

      // Deserialize the JSON into the defined struct
      let orders = deserialize_orders(&body).unwrap();

      Ok(orders)
  } else {
      Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
  }
}

pub async fn get_order_by_hash(hash: String) -> Result<Order, DexError> {
  let url = format!("{}?orderHash={}", UNISWAPX_API, hash);
  let response = reqwest::get(&url).await?;

  if response.status().is_success() {
    let body: String = response.text().await?;

    // Deserialize the JSON into the defined struct
    let orders = deserialize_orders(&body).unwrap();
    
    if orders.len() == 1 {
      Ok(orders[0].clone())
    } else {
      Err(DexError::Error("Order not found".to_owned()))
    }
  } else {
    Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
  }
}


fn deserialize_orders(json_str: &str) -> Result<Vec<Order>, serde_json::Error> {
  // Define a helper struct to match the JSON structure
  let data: Response = serde_json::from_str(json_str)?;
  Ok(data.orders)
}
