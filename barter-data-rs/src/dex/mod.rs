use core::fmt;
use thiserror::Error;

pub mod uniswapx;

pub mod tokens;

/// DEX Errors
#[derive(Debug, Error)]
pub enum DexError {
    Serde(serde_json::Error),
    Reqwest(reqwest::Error),
    UrlParseError(url::ParseError),
    Error(String),
}

impl From<serde_json::Error> for DexError {
    fn from(err: serde_json::Error) -> DexError {
        DexError::Serde(err)
    }
}

impl From<reqwest::Error> for DexError {
    fn from(err: reqwest::Error) -> DexError {
        DexError::Reqwest(err)
    }
}
// DexError formatter
impl fmt::Display for DexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DexError::Serde(ref err) => write!(f, "Serde error: {}", err),
            DexError::Reqwest(ref err) => write!(f, "Reqwest error: {}", err),
            DexError::Error(ref err) => write!(f, "Error: {}", err),
            DexError::UrlParseError(ref err) => write!(f, "UrlParseError: {}", err),
        }
    }
}
