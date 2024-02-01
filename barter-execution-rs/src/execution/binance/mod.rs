use std::{collections::HashMap, marker::PhantomData};

use async_trait::async_trait;
use barter_integration::model::{
    instrument::{symbol::Symbol, Instrument},
    Exchange,
};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    error::ExecutionError,
    fill::{Fees, FillEvent},
    model::{
        balance::SymbolBalance,
        execution_event::ExchangeRequest,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
        order_event::OrderEvent,
        AccountEvent,
    },
    ExecutionClient, ExecutionId,
};

use self::{
    connection::{BinanceApi, BinanceClient},
    requests::{FutOrderResponse, FUT_BALANCES_REQUEST},
};

pub mod connection;
pub mod requests;

/// Simulated [`ExecutionClient`] implementation that integrates with the Barter
/// [`SimulatedExchange`](super::exchange::SimulatedExchange).
#[derive(Debug)]
pub struct BinanceExecution<Event> {
    client: BinanceClient,
    instruments_map: HashMap<BinancePair, Instrument>,
    // client_type: BinanceApi,
    pub event_tx: mpsc::UnboundedSender<Event>,
    pub order_tx: mpsc::UnboundedSender<ExchangeRequest>,
}

/// Config for initializing a [`SimulatedExecution`] instance.
#[derive(Debug, Clone)]
pub struct BinanceConfig {
    client_type: BinanceApi,
    instruments: Vec<Instrument>,
    order_tx: mpsc::UnboundedSender<ExchangeRequest>,
}

#[async_trait]
impl<Event> ExecutionClient for BinanceExecution<Event>
where
    Event: Send + From<AccountEvent>,
{
    const CLIENT: ExecutionId = ExecutionId::Simulated;
    type Config = BinanceConfig;
    type Event = Event;

    async fn init(config: Self::Config, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        let client = BinanceClient::new(config.client_type);
        Self {
            client,
            instruments_map: Self::instruments_map(config.instruments),
            event_tx,
            order_tx: config.order_tx,
            // client_type: config.client_type,
        }
    }

    fn event_tx(&self) -> &mpsc::UnboundedSender<Event> {
        &self.event_tx
    }

    // async fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
    //     let result: FutOrderResponse = self.client.submit_order(order).await?;

    //     Ok(FillEvent {
    //         time: Utc::now(),
    //         exchange: order.exchange.clone(),
    //         instrument: order.instrument.clone(),
    //         market_meta: order.market_meta,
    //         decision: order.decision,
    //         quantity: order.quantity,
    //         fill_value_gross: result.cumQty,
    //         // TODO: compute fees
    //         fees: Fees {
    //             exchange: 0.0,
    //             slippage: 0.0,
    //             network: 0.0,
    //         },
    //     })
    // }

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

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BinancePair(String);

impl BinancePair {
    pub fn new(base: &Symbol, quote: &Symbol) -> Self {
        Self(format!("{base}{quote}").to_uppercase())
    }
}
impl<Event> BinanceExecution<Event> {
    fn instruments_map(instruments: Vec<Instrument>) -> HashMap<BinancePair, Instrument> {
        instruments
            .into_iter()
            .map(|instrument| {
                (
                    BinancePair::new(&instrument.base, &instrument.quote),
                    instrument,
                )
            })
            .collect()
    }
}
