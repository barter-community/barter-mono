use crate::cerebrum::exchange::ClientStatus;
use barter_data::event::{DataKind, MarketEvent};
use barter_execution::model::order::{Cancelled, Open, Order};
use barter_execution::model::AccountEvent;
use barter_execution::{error::ExecutionError, model::order::InFlight};
use barter_integration::model::{
    instrument::{symbol::Symbol, Instrument},
    Exchange, Side,
};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Market(MarketEvent<DataKind>),
    Account(AccountEvent),
    Command(Command),
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Terminate,
    FetchOpenPositions,
    ExitPosition,
    ExitAllPositions,
}

#[derive(Debug)]
pub struct EventFeed {
    pub event_rx: mpsc::UnboundedReceiver<Event>,
}

impl EventFeed {
    pub fn new(event_rx: mpsc::UnboundedReceiver<Event>) -> Self {
        Self { event_rx }
    }

    pub fn next(&mut self) -> Event {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => break event,
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => panic!("todo"),
            }
        }
    }
}

#[derive(Debug)]
pub struct Trade {
    pub id: TradeId,
    pub order_id: String,
    pub instrument: Instrument,
    pub side: Side,
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug)]
pub struct TradeId(pub String);

#[derive(Debug)]
pub struct SymbolBalance {
    pub symbol: Symbol,
    pub balance: Balance,
}

#[derive(Clone, Copy, Debug)]
pub struct Balance {
    pub total: f64,
    pub available: f64,
}
