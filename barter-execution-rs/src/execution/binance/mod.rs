use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{
    error::ExecutionError,
    fill::FillEvent,
    model::{
        balance::SymbolBalance,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
        order_event::OrderEvent,
        AccountEvent,
    },
    ExecutionClient, ExecutionId,
};

use self::{
    connection::{BinanceApi, BinanceClient},
    requests::{FUT_BALANCES_REQUEST, SPOT_BALANCES_REQUEST},
};

pub mod connection;
pub mod requests;

/// Simulated [`ExecutionClient`] implementation that integrates with the Barter
/// [`SimulatedExchange`](super::exchange::SimulatedExchange).
#[derive(Debug)]
pub struct BinanceExecution {
    client: BinanceClient,
    client_type: BinanceApi,
}

/// Config for initializing a [`SimulatedExecution`] instance.
#[derive(Debug)]
pub struct BinanceConfig {
    client: BinanceClient,
    client_type: BinanceApi,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Simulated;
    type Config = BinanceConfig;

    async fn init(config: Self::Config, _: mpsc::UnboundedSender<AccountEvent>) -> Self {
        Self {
            client: config.client,
            client_type: config.client_type,
        }
    }

    fn generate_fill(&self, _order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
        // Assume (for now) that all orders are filled at the market price

        // Ok(FillEvent {
        //     time: Utc::now(),
        //     exchange: order.exchange.clone(),
        //     instrument: order.instrument.clone(),
        //     market_meta: order.market_meta,
        //     decision: order.decision,
        //     quantity: order.quantity,
        //     fill_value_gross,
        //     fees: self.calculate_fees(&fill_value_gross),
        // })
        todo!()
    }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        todo!()
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        let request = match self.client_type {
            BinanceApi::Futures(_) => FUT_BALANCES_REQUEST,
            BinanceApi::Spot(_) => todo!(),
        };
        match self.client.send(request).await {
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
        // // Oneshot channel to communicate with the SimulatedExchange
        // let (response_tx, response_rx) = oneshot::channel();

        // // Send CancelOrdersAll request to the SimulatedExchange
        // self.request_tx
        //     .send(BinanceEvent::CancelOrdersAll(response_tx))
        //     .expect("SimulatedExchange is offline - failed to send CancelOrdersAll request");

        // // Receive CancelOrdersAll response from the SimulatedExchange
        // response_rx
        //     .await
        //     .expect("SimulatedExchange is offline - failed to receive CancelOrdersAll response")
    }
}
