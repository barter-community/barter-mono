use self::uni_order::{Response, UniOrder};
use super::tokens::TokenCache;
use super::DexError;
use crate::event::{DataKind, MarketEvent};
use crate::subscription::intent_order::{IntentOrder, IntentOrderUpdate};
use barter_integration::model::instrument::{kind::InstrumentKind, Instrument};
use eyre::Result;
use market::Market;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use reqwest;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::time::{sleep, Duration};

pub mod market;
pub mod uni_order;

const UNISWAPX_API: &str = "https://api.uniswap.org/v2/orders";

fn convert_bigint_string_to_float(
    bigint_str: &str,
    decimals: i32,
    radix: u32,
) -> Result<(f64), DexError> {
    let result = BigInt::parse_bytes(bigint_str.as_bytes(), radix);
    match result {
        Some(bigint) => {
            let float = bigint.to_f64().unwrap() / 10f64.powi(decimals);
            return Ok(float);
        }
        None => {
            return Err(DexError::Error("Failed to parse BigInt".to_owned()));
        }
    }
}

async fn map_uni_orders_to_intent_orders(
    uni_orders: Vec<UniOrder>,
    event: IntentOrderUpdate,
) -> Result<Vec<IntentOrder>, DexError> {
    let mut intent_orders = Vec::new();

    let tokens = TokenCache::instance().lock().await;
    for uni_order in uni_orders {
        // TODO: Why are there multiple output tokens?
        let token_in = tokens.get_token(&1, &uni_order.input.token).await?;
        let token_out = tokens.get_token(&1, &uni_order.outputs[0].token).await?;

        let market = Market::new(&token_in, &token_out);
        let (quote, base, buy) =
            market::Market::get_quote_and_base(&market.quotes, &token_in.symbol, &token_out.symbol);
        let instrument: Instrument =
            Instrument::new(quote.clone(), base.clone(), InstrumentKind::IntentOrder);

        let input_start_amt =
            convert_bigint_string_to_float(&uni_order.input.start_amount, token_in.decimals, 10)?;
        let output_start_amt = convert_bigint_string_to_float(
            &uni_order.outputs[0].start_amount,
            token_out.decimals,
            10,
        )?;
        let input_end_amt =
            convert_bigint_string_to_float(&uni_order.input.end_amount, token_in.decimals, 10)?;
        let output_end_amt = convert_bigint_string_to_float(
            &uni_order.outputs[0].end_amount,
            token_out.decimals,
            10,
        )?;

        // Handle orientation for instrument
        let amount = if market.buy {
            output_start_amt
        } else {
            input_start_amt
        };
        let start_ask = if market.buy {
            input_start_amt / output_start_amt
        } else {
            output_start_amt / input_start_amt
        };
        let end_ask = if market.buy {
            input_end_amt / output_end_amt
        } else {
            output_end_amt / input_end_amt
        };

        let intent_order = IntentOrder {
            id: uni_order.order_hash,
            event,
            instrument,
            amount: amount,
            amount_in: uni_order.input.start_amount,
            amount_out: uni_order.outputs[0].start_amount.clone(),
            start_ask,
            end_ask,
            price: start_ask,
            buy: market.buy,
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

// filter orders that don't already exist in self.open_orders, and remove old orders in open_orders
pub fn filter_open_orders(
    open_orders: &mut Vec<UniOrder>,
    new_orders: &Vec<UniOrder>,
) -> Vec<UniOrder> {
    let mut filtered_orders: Vec<UniOrder> = Vec::new();
    let mut exists_list: Vec<String> = Vec::new();

    for order in new_orders {
        // Check if the order already exists in open_orders
        if let Some(_) = open_orders
            .iter()
            .find(|&open_order| open_order.order_hash == order.order_hash)
        {
            // If it exists, check if it's not in exists_list and add it if necessary
            if !exists_list.contains(&order.order_hash) {
                exists_list.push(order.order_hash.clone());
            }
        } else {
            // If it does not exist, push to filtered_orders
            filtered_orders.push(order.clone());
        }
    }

    // Using exists_list remove the orders in open_orders that no longer exist (they have been filled or cancelled)
    let mut remove_list: Vec<usize> = Vec::new();
    for (i, open_order) in open_orders.iter().enumerate() {
        if !exists_list.contains(&open_order.order_hash) {
            remove_list.push(i);
        }
    }

    // Sort the indices in reverse order and remove duplicates
    let mut sorted_indices = remove_list.clone();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a)); // Sort in reverse
    sorted_indices.dedup(); // Remove duplicates
                            // Remove old orders
    for index in sorted_indices {
        if index < open_orders.len() {
            open_orders.remove(index);
        }
    }

    // return filtered orders
    return filtered_orders;
}

pub fn init() -> UnboundedReceiver<MarketEvent<DataKind>> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut open_orders = Vec::<UniOrder>::new();
        loop {
            let mut result = get_open_orders(1).await;
            match result {
                Ok(orders) => {
                    let mut new_orders = filter_open_orders(&mut open_orders, &orders);

                    if new_orders.len() > 0 {
                        // Convert to intent orders
                        let result = map_uni_orders_to_intent_orders(
                            new_orders.clone(),
                            IntentOrderUpdate::Opened,
                        )
                        .await;
                        match result {
                            Ok(intent_orders) => {
                                for order in &intent_orders {
                                    let _ = tx.send(MarketEvent::from(order));
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Error occurred mapping uni orders to intent orders! {}",
                                    e
                                );
                            }
                        }
                        open_orders.append(&mut new_orders);
                    }
                }
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
    return rx;
}

fn deserialize_orders(json_str: &str) -> Result<Vec<UniOrder>, serde_json::Error> {
    // Define a helper struct to match the JSON structure
    let data: Response = serde_json::from_str(json_str)?;
    Ok(data.orders)
}
