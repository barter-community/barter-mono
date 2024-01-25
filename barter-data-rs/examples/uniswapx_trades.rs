use barter_data::{
  dex::uniswapx::{UniswapX}
};


#[tokio::main]
async fn main() {



    let uni = UniswapX::new();
    let rx = uni.start();
}