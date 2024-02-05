use async_trait::async_trait;
use barter_integration::model::{instrument::symbol::Symbol, Exchange};

use crate::{
    error::ExecutionError,
    model::{
        balance::SymbolBalance,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
    },
    ExecutionClient, ExecutionId,
};

use self::{
    connection::{BinanceApi, BinanceClient},
    requests::FUT_BALANCES_REQUEST,
};

pub mod connection;
pub mod requests;

/// Binance [`ExecutionClient`] implementation that integrates with the Barter
#[derive(Debug)]
pub struct BinanceExecution {
    client: BinanceClient,
    // client_type: BinanceApi,
}

/// Config for initializing a [`BinanceExecution`] instance.
#[derive(Debug, Clone, Copy)]
pub struct BinanceConfig {
    client_type: BinanceApi,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    type Config = BinanceConfig;

    fn exchange(&self) -> Exchange {
        Exchange::from(ExecutionId::Simulated)
    }

    async fn init(config: Self::Config) -> Self {
        let client = BinanceClient::new(config.client_type);
        Self {
            client,
            // client_type: config.client_type,
        }
    }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        todo!()
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        match self.client.send(FUT_BALANCES_REQUEST).await {
            Ok(response) => {
                println!("{:#?}", response);
                return Ok(<Vec<SymbolBalance>>::from(response));
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(e);
            }
        }
    }

    async fn open_orders(
        &self,
        _open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        todo!()
    }

    async fn cancel_orders(
        &self,
        _cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>> {
        todo!()
    }

    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError> {
        todo!()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BinancePair(String);

impl BinancePair {
    pub fn new(base: &Symbol, quote: &Symbol) -> Self {
        Self(format!("{base}{quote}").to_uppercase())
    }
}
