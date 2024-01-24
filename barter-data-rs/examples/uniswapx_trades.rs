use barter_data::{
  dex::uniswapx::{get_order_by_hash},
};


#[tokio::main]
async fn main() {

  // let ret = uniswapx.getOpenOrders(1).await;
  let result = get_order_by_hash(String::from("0xC8F91A7DAC6AEB2172B9F5B5897F7721741BAC8DB082D942EB9E37D3D5FBCAB6")).await;

  match result {
    Ok(order) => {
        println!("{:?}", order.encoded_order);
    },
    Err(e) => {
        eprintln!("Error occurred!");
    } 
  }
}