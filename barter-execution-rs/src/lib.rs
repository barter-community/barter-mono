#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    // missing_docs
)]
#![allow(clippy::type_complexity)]

//! # Barter-Execution
//! High-performance and normalised trading interface capable of executing across many financial
//! venues. Also provides a feature rich simulated exchange to assist with backtesting
//! and dry-trading. Communicate with an exchange by initialising it's associated
//! `ExecutionClient` instance.
//! **It is:**
//! * **Easy**: ExecutionClient trait provides a unified and simple language for interacting with
//! exchanges.
//! * **Normalised**: Allow your strategy to communicate with every real or simulated exchange
//! using the same interface.
//! * **Extensible**: Barter-Execution is highly extensible, making it easy to contribute by adding
//! new exchange integrations!
//!
//! See `README.md` for more information and examples.

use crate::{
    error::ExecutionError,
    model::{
        balance::SymbolBalance,
        order::{Cancelled, Open, Order, OrderId, RequestCancel, RequestOpen},
        AccountEvent,
    },
};
use async_trait::async_trait;
use barter_integration::model::Exchange;
use fill::FillEvent;
use model::{execution_event::ExchangeRequest, order_event::OrderEvent, AccountEventKind};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tracing::error;

// Fill event
pub mod fill;

/// Errors generated during live, dry, or simulated execution.
pub mod error;

/// Core data structures to support executing on exchanges.
///
/// eg/ `Order`, `Balance`, `Trade` etc.
pub mod model;

/// [`ExecutionClient`] implementations for official exchanges.
pub mod execution;

/// Simulated Exchange and it's associated simulated [`ExecutionClient`].
pub mod simulated;

/// Defines the communication with the exchange. Each exchange integration requires it's own
/// implementation.
#[async_trait]
pub trait ExecutionClient {
    const CLIENT: ExecutionId;
    type Config;

    /// Initialise a new [`ExecutionClient`] with the provided [`Self::Config`] and
    /// [`AccountEvent`] transmitter.
    ///
    /// **Note:**
    /// Usually entails spawning an asynchronous WebSocket event loop to consume [`AccountEvent`]s
    /// from the exchange, as well as returning the HTTP client `Self`.
    async fn init(config: Self::Config, event_tx: mpsc::UnboundedSender<AccountEvent>) -> Self;

    /// Return a [`mpsc::UnboundedSender`] that is used to send [`OrderEvent`]s to the exchange.
    fn request_tx(&self) -> mpsc::UnboundedSender<ExchangeRequest>;

    /// Return a [`mpsc::UnboundedReceiver`] that is used to receive [`OrderEvent`]s from the
    fn event_tx(&self) -> mpsc::UnboundedSender<AccountEvent>;

    fn exchange(&self) -> Exchange;

    /// Return a [`FillEvent`] from executing the input [`OrderEvent`].
    // fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError>;

    /// Fetch account [`Order<Open>`]s.
    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError>;

    /// Fetch account [`SymbolBalance`]s.
    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError>;

    /// Open orders.
    async fn open_orders(
        &self,
        open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>>;

    /// Cancel [`Order<Open>`]s.
    async fn cancel_orders(
        &self,
        cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>>;

    /// Cancel all account [`Order<Open>`]s.
    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError>;

    fn send_account_tx(&self, kind: AccountEventKind) {
        let account_event = AccountEvent {
            exchange: self.exchange(),
            received_time: chrono::Utc::now(),
            kind,
        };
        // TODO how do we handle a send error?
        self.event_tx()
            .send(account_event)
            .expect("Execution engine is offline");
    }

    async fn run(&self, mut request_rx: mpsc::UnboundedReceiver<ExchangeRequest>) {
        // TODO: better handling of errors?
        while let Some(orders) = request_rx.recv().await {
            match orders {
                ExchangeRequest::OpenOrders(orders) => {
                    let open_orders = self.open_orders(orders).await;
                    let open_orders = filter_responses(open_orders);

                    let account_event = AccountEventKind::OrdersNew(open_orders);
                    self.send_account_tx(account_event);
                }
                ExchangeRequest::CancelOrders(order_events) => {
                    let cancelled_orders = self.cancel_orders(order_events).await;
                    let cancelled_orders = filter_responses(cancelled_orders);

                    let account_event = AccountEventKind::OrdersCancelled(cancelled_orders);
                    self.send_account_tx(account_event);
                }
                ExchangeRequest::CancelOrdersAll => {
                    match self.cancel_orders_all().await {
                        Ok(orders) => {
                            self.send_account_tx(AccountEventKind::OrdersCancelled(orders))
                        }
                        Err(e) => error!(error = ?e, "failed to cancel all orders"),
                    };
                }
                ExchangeRequest::FetchOrdersOpen => {
                    match self.fetch_orders_open().await {
                        Ok(orders) => self.send_account_tx(AccountEventKind::OrdersOpen(orders)),
                        Err(e) => error!(error = ?e, "failed to fetch open orders"),
                    };
                }
                ExchangeRequest::FetchBalances => {
                    match self.fetch_balances().await {
                        Ok(balances) => self.send_account_tx(AccountEventKind::Balances(balances)),
                        Err(e) => error!(error = ?e, "failed to fetch balances"),
                    };
                }
            }
        }
    }
}

pub fn filter_responses<T>(responses: Vec<Result<T, ExecutionError>>) -> Vec<T> {
    responses
        .into_iter()
        .filter_map(|response| match response {
            Ok(response) => Some(response),
            Err(e) => {
                error!(error = ?e, "failed to submit an order");
                None
            }
        })
        .collect::<Vec<T>>()
}

/// Unique identifier for an [`ExecutionClient`] implementation.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename = "execution", rename_all = "snake_case")]
pub enum ExecutionId {
    Simulated,
    Binance,
}

impl From<ExecutionId> for Exchange {
    fn from(execution_id: ExecutionId) -> Self {
        Exchange::from(execution_id.as_str())
    }
}

impl Display for ExecutionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ExecutionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionId::Simulated => "simulated",
            ExecutionId::Binance => "binance",
        }
    }
}

/// Utilities for generating common data structures required for testing.
pub mod test_util {
    use crate::{
        model::{
            trade::{SymbolFees, Trade, TradeId},
            ClientOrderId,
        },
        simulated::exchange::account::order::Orders,
        Open, Order, OrderId,
    };
    use barter_data::subscription::trade::PublicTrade;
    use barter_integration::model::{
        instrument::{kind::InstrumentKind, Instrument},
        Exchange, Side,
    };

    pub fn client_orders(
        trade_number: u64,
        bids: Vec<Order<Open>>,
        asks: Vec<Order<Open>>,
    ) -> Orders {
        Orders {
            trade_counter: trade_number,
            bids,
            asks,
        }
    }

    pub fn order_open(
        cid: ClientOrderId,
        side: Side,
        price: f64,
        quantity: f64,
        filled: f64,
    ) -> Order<Open> {
        Order {
            exchange: Exchange::from("exchange"),
            instrument: Instrument::from(("base", "quote", InstrumentKind::Perpetual)),
            cid,
            side,
            state: Open {
                id: OrderId::from("order_id"),
                price,
                quantity,
                filled_quantity: filled,
            },
        }
    }

    pub fn public_trade(side: Side, price: f64, amount: f64) -> PublicTrade {
        PublicTrade {
            id: "trade_id".to_string(),
            price,
            amount,
            side,
        }
    }

    pub fn trade(id: TradeId, side: Side, price: f64, quantity: f64, fees: SymbolFees) -> Trade {
        Trade {
            id,
            order_id: OrderId::from("order_id"),
            instrument: Instrument::from(("base", "quote", InstrumentKind::Perpetual)),
            side,
            price,
            quantity,
            fees,
        }
    }
}
