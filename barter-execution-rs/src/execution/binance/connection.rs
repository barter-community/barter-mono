use std::{fmt::Debug, marker::PhantomData};

use bytes::Bytes;

use barter_integration::{
    error::SocketError,
    protocol::http::{
        private::{encoder::HexEncoder, get_default_signer, RequestSigner, Signer},
        rest::{client::RestClient, ApiRequest, QueryParams, RestRequest},
        HttpParser,
    },
};
use chrono::Utc;
use dotenv::dotenv;
use hmac::Hmac;
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    error::ExecutionError,
    fill::Decision,
    model::order_event::{OrderEvent, OrderExecutionType, OrderType},
};

use super::requests::BalancesResponse;

#[derive(Debug, Copy, Clone)]
pub enum LiveOrTest {
    Live,
    Test,
}

#[derive(Debug, Copy, Clone)]
pub enum BinanceApi {
    Spot(LiveOrTest),
    Futures(LiveOrTest),
}

pub type BinanceInternalClient =
    RestClient<RequestSigner<BinanceSigner, Hmac<sha2::Sha256>, HexEncoder>, BinanceParser>;

#[derive(Debug)]
pub struct BinanceClient {
    pub client: BinanceInternalClient,
    pub kind: BinanceApi,
}

impl BinanceClient {
    pub fn new(api_type: BinanceApi) -> BinanceClient {
        dotenv().ok();
        let api_key = std::env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY must be set.");
        let api_secret = std::env::var("BINANCE_SECRET").expect("BINANCE_SECRET must be set.");

        // // Construct Metric channel to send Http execution metrics over
        let (http_metric_tx, _http_metric_rx) = mpsc::unbounded_channel();

        let request_signer = get_default_signer(
            &api_secret,
            BinanceSigner {
                api_key: api_key.to_string(),
                timestamp_delta: 0,
            },
        );

        let client_url = match api_type {
            BinanceApi::Spot(_) => "https://api.binance.com",
            BinanceApi::Futures(kind) => match kind {
                LiveOrTest::Live => "https://fapi.binance.com",
                LiveOrTest::Test => "https://testnet.binancefuture.com",
            },
        };

        // Build RestClient with Binance configuration
        let client = RestClient::new(
            client_url.to_string(),
            http_metric_tx,
            request_signer,
            BinanceParser,
        );
        BinanceClient {
            client,
            kind: api_type,
        }
    }
    pub async fn send<Request>(&self, request: Request) -> Result<Request::Response, ExecutionError>
    where
        Request: RestRequest,
        <Request as RestRequest>::Response: Debug,
    {
        self.client.execute(request).await
    }

    pub(super) async fn create_order<Response>(
        &self,
        order: &OrderEvent,
    ) -> Result<Response, ExecutionError>
    where
        Response: for<'de> Deserialize<'de> + Debug,
    {
        let instrument = &order.instrument;
        let symbol = format!("{}{}", instrument.base, instrument.quote).to_uppercase();
        let mut query_params = QueryParams::new();

        query_params.add_kv("symbol", symbol);
        // TODO better side logic?
        query_params.add_kv("side", get_order_side(order.decision));
        query_params.add_kv("quantity", order.quantity);
        // TODO should we generate our own?
        // builder.add_kv("newClientOrderId", &header.client_order_id);

        match order.order_type {
            OrderType::Limit {
                price,
                execution_type,
            } => {
                match execution_type {
                    OrderExecutionType::None => {
                        query_params.add_kv("type", "LIMIT");
                        query_params.add_kv("timeInForce", "GTC");
                    }
                    OrderExecutionType::MakerOnly => query_params.add_kv("type", "LIMIT_MAKER"),
                }
                query_params.add_kv("price", price);
            }
            OrderType::Market => query_params.add_kv("type", "MARKET"),
            OrderType::StopLoss { stop_price } => {
                query_params.add_kv("type", "STOP_LOSS");
                query_params.add_kv("stopPrice", stop_price);
                query_params.add_kv("timeInForce", "GTC");
            }
            OrderType::TrailingStop {
                trailing_delta,
                stop_price,
            } => {
                query_params.add_kv("type", "STOP_LOSS");
                query_params.add_kv("trailingDelta", trailing_delta);
                query_params.add_kv("timeInForce", "GTC");

                if let Some(stop_price) = stop_price {
                    query_params.add_kv("stopPrice", stop_price)
                }
            }
            _ => todo!("Order type not supported"),
        }
        let request: ApiRequest<Response, ()> = ApiRequest {
            path: "/sapi/v3/asset/getUserAsset",
            method: reqwest::Method::POST,
            tag_method: "fetch_balances",
            body: None,
            query_params: Some(query_params),
            response: PhantomData,
        };

        self.client.execute(request).await
    }
}

pub(super) fn get_order_side(side: Decision) -> &'static str {
    match side {
        Decision::Long => "BUY",
        Decision::Short => "SELL",
        Decision::CloseLong => "SELL",
        Decision::CloseShort => "BUY",
    }
}

#[derive(Debug)]
pub struct BinanceSigner {
    pub api_key: String,
    pub timestamp_delta: i64,
}

impl BinanceSigner {
    pub fn init(api_key: String, timestamp_delta: i64) -> Self {
        Self {
            api_key,
            timestamp_delta,
        }
    }
}

#[derive(Debug)]
pub struct BinanceSignConfig<'a> {
    api_key: &'a str,
    query_string: String,
}

impl Signer for BinanceSigner {
    type Config<'a> = BinanceSignConfig<'a> where Self: 'a;

    fn config<'a, Request>(
        &'a self,
        _: &Request,
        mut builder: RequestBuilder,
    ) -> Result<(Self::Config<'a>, RequestBuilder), SocketError>
    where
        Request: RestRequest,
    {
        let timestamp = (Utc::now().timestamp_millis() - self.timestamp_delta) as u128;

        // this is a little ugly, but the only way I could find to add
        // and grab query parameters to a request
        builder = builder.query(&[("timestamp", timestamp)]);
        let (client, request) = builder.build_split();
        if let Err(e) = request {
            return Err(SocketError::from(e));
        }
        let request = request.unwrap();
        let query_string = (&request).url().query().unwrap_or("").to_string();
        let builder = RequestBuilder::from_parts(client, request);

        Ok((
            BinanceSignConfig {
                api_key: self.api_key.as_str(),
                query_string,
            },
            builder,
        ))
    }

    fn bytes_to_sign<'a>(config: &Self::Config<'a>) -> Bytes {
        Bytes::copy_from_slice(format!("{}", config.query_string).as_bytes())
    }

    fn build_signed_request<'a>(
        config: Self::Config<'a>,
        builder: RequestBuilder,
        signature: String,
    ) -> Result<reqwest::Request, SocketError> {
        // Add Binance required Headers & build reqwest::Request
        builder
            .header("X-MBX-APIKEY", config.api_key)
            .query(&[("signature", &signature)])
            .build()
            .map_err(SocketError::from)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BinanceParser;

impl HttpParser for BinanceParser {
    type ApiError = serde_json::Value;
    type OutputError = ExecutionError;

    fn parse_api_error(&self, status: StatusCode, api_error: Self::ApiError) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        let error = api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            }
            _ => ExecutionError::Socket(SocketError::HttpResponse(status, error)),
        }
    }
}
