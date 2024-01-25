// pub struct RestHeadersBinance {
//     pub api_key: String,
//     pub is_usd_m_futures: bool,
// }

use bytes::Bytes;

use barter_integration::{
    error::SocketError,
    metric::Tag,
    model::instrument::symbol::Symbol,
    protocol::http::{private::Signer, rest::RestRequest, HttpParser},
};
use chrono::{DateTime, Utc};
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;
use thiserror::Error;

pub struct BinanceSigner {
    pub api_key: String,
}

pub struct BinanceSignConfig<'a> {
    api_key: &'a str,
    time: DateTime<Utc>,
    method: reqwest::Method,
    path: &'static str,
}

impl Signer for BinanceSigner {
    type Config<'a> = BinanceSignConfig<'a> where Self: 'a;

    fn config<'a, Request>(
        &'a self,
        _: Request,
        _: &RequestBuilder,
    ) -> Result<Self::Config<'a>, SocketError>
    where
        Request: RestRequest,
    {
        Ok(BinanceSignConfig {
            api_key: self.api_key.as_str(),
            time: Utc::now(),
            method: Request::method(),
            path: Request::path(),
        })
    }

    fn bytes_to_sign<'a>(config: &Self::Config<'a>) -> Bytes {
        Bytes::copy_from_slice(
            format!("{}{}{}", config.time, config.method, config.path).as_bytes(),
        )
    }

    fn build_signed_request<'a>(
        config: Self::Config<'a>,
        builder: RequestBuilder,
        signature: String,
    ) -> Result<reqwest::Request, SocketError> {
        // Add Ftx required Headers & build reqwest::Request
        builder
            .header("FTX-KEY", config.api_key)
            .header("FTX-TS", &config.time.timestamp_millis().to_string())
            .header("FTX-SIGN", &signature)
            .build()
            .map_err(SocketError::from)
    }
}

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

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}

pub struct FetchBalancesRequest;

impl RestRequest for FetchBalancesRequest {
    type Response = FetchBalancesResponse; // Define Response type
    type QueryParams = (); // FetchBalances does not require any QueryParams
    type Body = (); // FetchBalances does not require any Body

    fn path() -> &'static str {
        "/api/wallet/balances"
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }

    fn metric_tag() -> Tag {
        Tag::new("method", "fetch_balances")
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FetchBalancesResponse {
    success: bool,
    result: Vec<FtxBalance>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FtxBalance {
    #[serde(rename = "coin")]
    symbol: Symbol,
    total: f64,
}
