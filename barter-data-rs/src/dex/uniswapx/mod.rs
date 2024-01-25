
// use crate::event::{DataKind, MarketEvent};
// use barter_integration::model::{
//     instrument::{kind::InstrumentKind, Instrument}
// };
use eyre::Result;
// use serde::Deserialize;
// use std::sync::Arc;
// use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
// use tracing::{debug, info};
use reqwest;
// use std::collections::HashMap;
// use std::error::Error;
use self::order::{Order, Response};
use super::DexError;
use tokio::time::{Duration, sleep};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use std::sync::{Arc, Mutex,};


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


// filter orders that don't already exist in self.open_orders
pub fn filter_open_orders(open_orders: &Vec<Order>, new_orders: &Vec<Order>) -> Vec<Order> {
  let mut filtered_orders: Vec<Order> = Vec::new();

  for order in new_orders {
    // us the order.order_hash to check if the order already exists in self.open_orders
    let mut exists = false;
    for open_order in open_orders {
      if order.order_hash == open_order.order_hash {
        exists = true;
        break;
      }
    }

    if !exists {
      filtered_orders.push(order.clone());
    }
  }

  // return filtered orders
  return filtered_orders
}

#[derive(Debug, Clone)]
pub struct UniswapX {
  open_orders: Arc<Mutex<Vec<Order>>>,
}

impl UniswapX {
  pub fn new() -> Self {
    UniswapX {
      open_orders: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn start(&self) -> UnboundedReceiver<Vec<Order>> {
    let (tx, rx) = mpsc::unbounded_channel();
    let open_orders = self.open_orders.clone();

    tokio::spawn(async move {
        let mut result = get_open_orders(1).await;        
        match result {
          Ok(orders) => {
            let mut unlocked = open_orders.lock().unwrap().to_vec();
            let mut new_orders = filter_open_orders(&unlocked, &orders);

            if new_orders.len() > 0 {
              // Apped the new orders to the open_orders vec
              unlocked.append(&mut new_orders);
              let _ = tx.send(new_orders);
            }

          },
          Err(e) => {
              // Print dex error;              
              eprintln!("Error occurred getting open orders! {}", e);
          } 
        }      

        // Delay for 1 second
        let delay_duration = Duration::from_secs(2);
        sleep(delay_duration).await;
    });
    return rx
  }
  


}






fn deserialize_orders(json_str: &str) -> Result<Vec<Order>, serde_json::Error> {
  // Define a helper struct to match the JSON structure
  let data: Response = serde_json::from_str(json_str)?;
  Ok(data.orders)
}
