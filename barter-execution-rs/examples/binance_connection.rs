use barter_execution::execution::binance::execution::{
    connection::{BinanceParser, BinanceSigner},
    requests::{BALANCES_REQUEST, FUT_BALANCES},
};
use barter_integration::protocol::http::{
    private::{encoder::HexEncoder, RequestSigner},
    rest::client::RestClient,
};
use dotenv::dotenv;
use hmac::{digest::KeyInit, Hmac};
use tokio::sync::mpsc;

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("BINANCE_API_KEY").expect("WSS_URL must be set.");
    let api_secret = std::env::var("BINANCE_SECRET").expect("WSS_URL must be set.");

    // Construct Metric channel to send Http execution metrics over
    let (http_metric_tx, _http_metric_rx) = mpsc::unbounded_channel();

    // HMAC-SHA256 encoded account API secret used for signing private http requests
    let mac: Hmac<sha2::Sha256> = Hmac::new_from_slice(api_secret.as_bytes()).unwrap();

    // Build Binance configured RequestSigner for signing http requests with hex encoding
    let request_signer = RequestSigner::new(
        BinanceSigner {
            api_key: api_key.to_string(),
            timestamp_delta: 0,
        },
        mac,
        HexEncoder,
    );

    // Build RestClient with Binance configuration
    let rest_client = RestClient::new(
        // "https://api.binance.com",
        "https://fapi.binance.com",
        http_metric_tx,
        request_signer,
        BinanceParser,
    );

    match rest_client.execute(FUT_BALANCES).await {
        Ok(response) => println!("{:#?}", response),
        Err(e) => println!("{:?}", e),
    }
}
