
use eyre::Result;
use reqwest;
use crate::subscription::intent_order::{IntentOrder, IntentOrderUpdate};

use self::uni_order::{UniOrder, Response};
use super::tokens::TokenCache;
use super::DexError;
use tokio::time::{Duration, sleep};
use tokio::sync::mpsc::{self, UnboundedReceiver};

pub mod uni_order;

const UNISWAPX_API: &str = "https://api.uniswap.org/v2/orders";

async fn map_uni_orders_to_intent_orders(uni_orders: Vec<UniOrder>, event: IntentOrderUpdate) -> Result<Vec<IntentOrder>, DexError> {
  let mut intent_orders = Vec::new();

  let tokens = TokenCache::instance().lock().await;
  // tokio::pin!(tokens);
  // let token = tokens.get_token(&1, &String::from("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984").to_string()).await.unwrap();
  for uni_order in uni_orders {
    // TODO: Why are there multiple output tokens?
    let token_in = tokens.get_token(&1, &uni_order.input.token).await?;
    let token_out = tokens.get_token(&1, &uni_order.outputs[0].token).await?;
    // TODO: Fetch decimals for tokens from the token list
    let start_ask = uni_order.outputs[0].start_amount.parse::<f64>().unwrap() / uni_order.input.start_amount.parse::<f64>().unwrap();
    let end_ask = uni_order.outputs[0].end_amount.parse::<f64>().unwrap() / uni_order.input.start_amount.parse::<f64>().unwrap();
    let price = uni_order.outputs[0].end_amount.parse::<f64>().unwrap() / uni_order.input.start_amount.parse::<f64>().unwrap();

    let intent_order = IntentOrder {
        event,
        id: uni_order.order_hash.clone(), 
        in_token: token_in.symbol.clone(),
        in_token_addr: token_in.addr.clone(),
        in_amount: uni_order.input.start_amount.parse::<f64>().unwrap_or(0.0),
        out_token: token_out.symbol.clone(), 
        out_token_addr: token_out.addr.clone(), 
        out_amount: uni_order.outputs[0].end_amount.parse::<f64>().unwrap_or(0.0),
        start_ask,
        end_ask,
        price,     
        created_at: uni_order.created_at,
        order_type: uni_order.order_type.clone(),
        signature: uni_order.signature.clone(),
        encoded_order: uni_order.encoded_order.clone(),
    };

    intent_orders.push(intent_order);
  }

  Ok(intent_orders)
}

pub async fn get_open_orders(chainId: u8) -> Result<Vec<UniOrder>, DexError> {
  let url = format!("{}?chainId={}&orderStatus=open", UNISWAPX_API, chainId);
  let response = reqwest::get(&url).await?;

  if response.status().is_success() {
      let body: String = response.text().await?;
      // print body

      // Deserialize the JSON into the defined struct
      let orders: Vec<UniOrder> = deserialize_orders(&body).unwrap();

      Ok(orders)
  } else {
      Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
  }
}

pub async fn get_order_by_hash(hash: String) -> Result<UniOrder, DexError> {
  let url = format!("{}?orderHash={}", UNISWAPX_API, hash);
  let response = reqwest::get(&url).await?;

  if response.status().is_success() {
    let body: String = response.text().await?;

    // Deserialize the JSON into the defined struct
    let orders = deserialize_orders(&body).unwrap();
    
    if orders.len() == 1 {
      Ok(orders[0].clone())
    } else {
      Err(DexError::Error("UniOrder not found".to_owned()))
    }
  } else {
    Err(DexError::Reqwest(response.error_for_status().unwrap_err()))
  }
}


// filter orders that don't already exist in self.open_orders
pub fn filter_open_orders(open_orders: &Vec<UniOrder>, new_orders: &Vec<UniOrder>) -> Vec<UniOrder> {
  let mut filtered_orders: Vec<UniOrder> = Vec::new();

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
pub struct UniswapX {}

impl UniswapX {
  pub fn new() -> Self {
    UniswapX {
    }
  }

  pub fn test(&self) -> () {
    tokio::spawn(async move {
      let mut open_orders = Vec::<UniOrder>::new();
      loop {
        let tokens = TokenCache::instance().lock().await;
        let result = tokens.get_token(&1, &String::from("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")).await;
        match result {
          Ok(token) => {
            println!("Got Token {}", token);
          },
          Err(e) => {
            eprintln!("Error occurred getting token! {}", e);
          } 
        }
      }
    });
    ()
  }

  pub fn start(&self) -> UnboundedReceiver<Vec<IntentOrder>> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
      let mut open_orders = Vec::<UniOrder>::new();
      loop {       
        let mut result = get_open_orders(1).await;
        match result {
          Ok(orders) => {
            let mut new_orders = filter_open_orders(&open_orders, &orders);

            // TODO - delete the orders that no longer exist.
            if new_orders.len() > 0 {
              // Convert to intent orders
              let result = map_uni_orders_to_intent_orders(
                new_orders.clone(), 
                IntentOrderUpdate::Opened
              ).await;
              match result {
                Ok(intent_orders) => {
                  let _ = tx.send(intent_orders);
                },
                Err(e) => {
                  eprintln!("Error occurred mapping uni orders to intent orders! {}", e);
                } 
              }
              open_orders.append(&mut new_orders);
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
      } 
    });
    return rx
  }


}

fn deserialize_orders(json_str: &str) -> Result<Vec<UniOrder>, serde_json::Error> {
  // Define a helper struct to match the JSON structure
  let data: Response = serde_json::from_str(json_str)?;
  Ok(data.orders)
}
