use super::DexError;
use core::fmt;
use ethers::{
    contract::{abigen, ContractError},
    core::types::Address,
    providers::{Http, Provider},
};
use lazy_static::lazy_static;
use redis::{Client, Commands, RedisError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TokenCache {
    client: Client,
}

abigen!(
    IERC20,
    r#"[
    function decimals() external view returns (uint8)
    function symbol() external view returns (string memory)
  ]"#,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub addr: String,
    pub symbol: String,
    pub decimals: u8,
}

lazy_static! {
    static ref REDIS_CACHE: Mutex<TokenCache> = Mutex::new(TokenCache {
        client: Client::open("redis://127.0.0.1/").expect("Failed to create Redis client"),
    });
}

impl TokenCache {
    pub fn instance() -> &'static Mutex<TokenCache> {
        &REDIS_CACHE
    }

    pub async fn get_token(&self, chain_id: &u64, address: &String) -> Result<Token, DexError> {
        let mut con = self.client.get_connection()?;
        let key = format!("{}:{}", chain_id, address);
        let result: Result<String, RedisError> = con.get(&key);
        match result {
            Ok(value) => {
                let token: Token = serde_json::from_str(&value)?;
                return Ok(token);
            }
            Err(e) => {
                // the token doesn't exist yet
                let token = self.get_token_from_chain(chain_id, address).await?;
                con.set(&key, serde_json::to_string(&token)?)?;
                return Ok(token);
            }
        }
    }

    pub async fn get_token_from_chain(
        &self,
        chain_id: &u64,
        address: &String,
    ) -> Result<Token, DexError> {
        // Special Cases
        if address.to_lowercase() == "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_lowercase() {
            let token = Token {
                addr: address.to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            };
            return Ok(token);
        } else if address.to_lowercase() == "0x0000000000000000000000000000000000000000" {
            let token = Token {
                addr: address.to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            };
            return Ok(token);
        } else if address.to_lowercase()
            == "0x0d88ed6e74bbfd96b831231638b66c05571e824f".to_lowercase()
        {
            let token = Token {
                addr: address.to_string(),
                symbol: "AVT".to_string(),
                decimals: 18,
            };
            return Ok(token);
        }

        // Connect to the network

        let rpc_url = std::env::var("ETH_NODE_URL").expect("WSS_URL must be set.");
        let provider: Provider<Http> = Provider::<Http>::try_from(rpc_url)?;

        // Create an instance of the ERC20 contract
        let addr: Address = match address.parse() {
            Ok(addr) => addr,
            Err(e) => return Err(DexError::Error(e.to_string())),
        };

        let client = Arc::new(provider);
        let erc20 = IERC20::new(addr, client);

        let decimals = erc20.decimals().call().await?;
        let symbol = erc20.symbol().call().await?;
        let token = Token {
            addr: address.to_string(),
            symbol: symbol,
            decimals: decimals,
        };
        Ok(token)
    }
}
