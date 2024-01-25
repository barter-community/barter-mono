use barter_execution::execution::binance::execution::connection::{
    BinanceParser, BinanceSigner, FetchBalancesRequest,
};
use barter_integration::protocol::http::{
    private::{encoder::HexEncoder, RequestSigner},
    rest::client::RestClient,
};
use hmac::{digest::KeyInit, Hmac};
use tokio::sync::mpsc;

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    // Construct Metric channel to send Http execution metrics over
    let (http_metric_tx, _http_metric_rx) = mpsc::unbounded_channel();

    // HMAC-SHA256 encoded account API secret used for signing private http requests
    let mac: Hmac<sha2::Sha256> = Hmac::new_from_slice("api_secret".as_bytes()).unwrap();

    // Build Ftx configured RequestSigner for signing http requests with hex encoding
    let request_signer = RequestSigner::new(
        BinanceSigner {
            api_key: "api_key".to_string(),
        },
        mac,
        HexEncoder,
    );

    // Build RestClient with Ftx configuration
    let rest_client = RestClient::new(
        "https://ftx.com",
        http_metric_tx,
        request_signer,
        BinanceParser,
    );

    // Fetch Result<FetchBalancesResponse, ExecutionError>
    let _response = rest_client.execute(FetchBalancesRequest).await;
}
