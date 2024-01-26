use barter_data::dex::{
    tokens::TokenCache,
    uniswapx::{get_open_orders, UniswapX},
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let uni = UniswapX::new();
    let mut rx = uni.select();

    loop {
        let result = rx.recv().await;
        match result {
            Some(orders) => {
                for order in orders {
                    println!("Main - New Order: {:?}", order.id);
                }
            }
            None => {
                println!("No orders - something has failed");
            }
        }
    }
}
