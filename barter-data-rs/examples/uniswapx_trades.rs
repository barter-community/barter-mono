use barter_data::{
  dex::uniswapx::{UniswapX, get_open_orders}
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
          println!("Main - New Orders: {:?}", orders);
        },
        None => {
          println!("No orders");
        }
      }
    }
}