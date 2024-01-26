use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::{mpsc, oneshot};

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

use super::BinanceEvent;
pub mod connection;
pub mod requests;

/// Simulated [`ExecutionClient`] implementation that integrates with the Barter
/// [`SimulatedExchange`](super::exchange::SimulatedExchange).
#[derive(Clone, Debug)]
pub struct BinanceExecution {
    /// Simulated fee percentage to be used for each [`Fees`] field in decimal form (eg/ 0.01 for 1%)
    pub request_tx: mpsc::UnboundedSender<BinanceEvent>,
}

/// Config for initializing a [`SimulatedExecution`] instance.
#[derive(Clone, Debug)]
pub struct BinanceConfig {
    /// Simulated fee percentage to be used for each [`Fees`] field in decimal form (eg/ 0.01 for 1%)
    pub request_tx: mpsc::UnboundedSender<BinanceEvent>,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Simulated;
    type Config = BinanceConfig;

    async fn init(config: Self::Config, _: mpsc::UnboundedSender<AccountEvent>) -> Self {
        let BinanceConfig { request_tx } = config;
        Self { request_tx }
    }

    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
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
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send FetchOrdersOpen request to the SimulatedExchange
        self.request_tx
            .send(BinanceEvent::FetchOrdersOpen(response_tx))
            .expect("SimulatedExchange is offline - failed to send FetchOrdersOpen request");

        // Receive FetchOrdersOpen response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive FetchOrdersOpen response")
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send FetchBalances request to the SimulatedExchange
        self.request_tx
            .send(BinanceEvent::FetchBalances(response_tx))
            .expect("SimulatedExchange is offline - failed to send FetchBalances request");

        // Receive FetchBalances response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive FetchBalances response")
    }

    async fn open_orders(
        &self,
        open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send OpenOrders request to the SimulatedExchange
        self.request_tx
            .send(BinanceEvent::OpenOrders((open_requests, response_tx)))
            .expect("SimulatedExchange is offline - failed to send OpenOrders request");

        // Receive OpenOrders response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive OpenOrders response")
    }

    async fn cancel_orders(
        &self,
        cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send CancelOrders request to the SimulatedExchange
        self.request_tx
            .send(BinanceEvent::CancelOrders((cancel_requests, response_tx)))
            .expect("SimulatedExchange is offline - failed to send CancelOrders request");

        // Receive CancelOrders response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive CancelOrders response")
    }

    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send CancelOrdersAll request to the SimulatedExchange
        self.request_tx
            .send(BinanceEvent::CancelOrdersAll(response_tx))
            .expect("SimulatedExchange is offline - failed to send CancelOrdersAll request");

        // Receive CancelOrdersAll response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive CancelOrdersAll response")
    }
}
