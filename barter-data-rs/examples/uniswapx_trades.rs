use barter_data::dex::uniswapx;
use dotenv::dotenv;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut rx = uniswapx::select();

    loop {
        let result = rx.recv().await;
        match result {
            Some(order) => {
                println!("Main - New Order: {:?}", order.kind);
            }
            None => {
                println!("No orders - something has failed");
            }
        }
    }
}
