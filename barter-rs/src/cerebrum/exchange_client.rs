use async_trait::async_trait;
use barter_execution::{
    error::ExecutionError,
    execution::binance::{BinanceConfig, BinanceExecution},
    model::{
        balance::SymbolBalance,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
    },
    simulated::execution::{SimulatedExecution, SimulationConfig},
    ExecutionClient,
};
use barter_integration::model::Exchange;

// Todo:
//   - Better name for this? This is the equivilant to ExchangeId...
//    '--> renamed to ClientId for now to avoid confusion in development
#[derive(Debug)]
pub enum ClientId {
    Simulated(SimulationConfig),
    Binance(BinanceConfig),
}

#[derive(Debug)]
pub enum ExchangeClient {
    Simulated(SimulatedExecution),
    Binance(BinanceExecution),
}

#[async_trait]
impl ExecutionClient for ExchangeClient {
    type Config = ClientId;

    fn exchange(&self) -> Exchange {
        match self {
            ExchangeClient::Simulated(client) => client.exchange(),
            ExchangeClient::Binance(client) => client.exchange(),
        }
    }

    async fn init(config: Self::Config) -> Self {
        match config {
            ClientId::Simulated(config) => {
                let client = SimulatedExecution::init(config).await;
                ExchangeClient::Simulated(client)
            }
            ClientId::Binance(config) => {
                let client = BinanceExecution::init(config).await;
                ExchangeClient::Binance(client)
            }
        }
    }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        match self {
            ExchangeClient::Simulated(client) => client.fetch_orders_open().await,
            ExchangeClient::Binance(client) => client.fetch_orders_open().await,
        }
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        match self {
            ExchangeClient::Simulated(client) => client.fetch_balances().await,
            ExchangeClient::Binance(client) => client.fetch_balances().await,
        }
    }

    async fn open_orders(
        &self,
        open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        match self {
            ExchangeClient::Simulated(client) => client.open_orders(open_requests).await,
            ExchangeClient::Binance(client) => client.open_orders(open_requests).await,
        }
    }

    async fn cancel_orders(
        &self,
        cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>> {
        match self {
            ExchangeClient::Simulated(client) => client.cancel_orders(cancel_requests).await,
            ExchangeClient::Binance(client) => client.cancel_orders(cancel_requests).await,
        }
    }

    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError> {
        match self {
            ExchangeClient::Simulated(client) => client.cancel_orders_all().await,
            ExchangeClient::Binance(client) => client.cancel_orders_all().await,
        }
    }
}
