use barter_data::{
  dex::uniswapx::{UniswapX},
};


#[tokio::main]
async fn main() {

  // initialise uniswapx
  let uniswapx = UniswapX::new().await;
  // let ret = uniswapx.getOpenOrders(1).await;
  let result = uniswapx.getOrderHash(String::from("0xC8F91A7DAC6AEB2172B9F5B5897F7721741BAC8DB082D942EB9E37D3D5FBCAB6")).await;

  match result {
    Ok(order) => {
        println!("{:?}", order.encodedOrder);
    },
    Err(e) => {
        eprintln!("Error occurred!");
    } 
  }
}