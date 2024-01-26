use barter_data::{
  dex::{tokens::TokenCache, uniswapx::{UniswapX, get_open_orders}}
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let uni = UniswapX::new();
    let mut rx = uni.start();

    loop {
      let result = rx.recv().await;
      match result {
        Some(orders) => {
          for order in orders {
            println!("Main - New Order: {:?}", order.id);
          }
        },
        None => {
          println!("No orders - something has failed");
        }
      }
    }
}