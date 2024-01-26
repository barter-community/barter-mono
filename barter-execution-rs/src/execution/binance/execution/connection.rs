use bytes::Bytes;

use barter_integration::{
    error::SocketError,
    protocol::http::{private::Signer, rest::RestRequest, HttpParser},
};
use chrono::Utc;
use reqwest::{RequestBuilder, StatusCode};
use thiserror::Error;

#[derive(Debug)]
pub struct BinanceSigner {
    pub api_key: String,
    pub timestamp_delta: i64,
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
        builder = builder.query(&[("timestamp", format!("{}", timestamp).as_str())]);
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

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}
