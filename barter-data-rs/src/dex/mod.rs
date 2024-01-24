

pub mod uniswapx;


/// DEX Errors
#[derive(Debug)]
pub enum DexError {
  Serde(serde_json::Error),
  Reqwest(reqwest::Error),
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