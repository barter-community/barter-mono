use ethers::{
    contract::ContractError,
    providers::{Http, Provider},
    utils::hex::FromHexError,
};
use thiserror::Error;

pub mod uniswapx;

pub mod tokens;

/// DEX Errors
#[derive(Debug, Error)]
pub enum DexError {
    #[error("Error: {0}")]
    Error(String),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("UrlParseError: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("UrlParseError: {0}")]
    FromHexError(#[from] FromHexError),
    #[error("ContractError: {0}")]
    ContractError(#[from] ContractError<Provider<Http>>),
    #[error("RedisError: {0}")]
    RedisError(#[from] redis::RedisError),
}
