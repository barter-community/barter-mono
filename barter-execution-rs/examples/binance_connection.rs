use barter_execution::execution::binance::{
    connection::BinanceApi,
    connection::{BinanceClient, LiveOrTest},
    requests::FUT_BALANCES_REQUEST,
};

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    // Build RestClient with Binance configuration
    let rest_client = BinanceClient::new(BinanceApi::Futures(LiveOrTest::Live));

    // can also try BALANCES_REQUEST
    match rest_client.send(FUT_BALANCES_REQUEST).await {
        Ok(response) => println!("{:#?}", response),
        Err(e) => println!("{:?}", e),
    }
}
