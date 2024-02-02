use barter_data::event::{DataKind, MarketEvent};
use barter_execution::model::AccountEvent;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Market(MarketEvent<DataKind>),
    Account(AccountEvent),
    Command(Command),
}

impl From<AccountEvent> for Event {
    fn from(account_event: AccountEvent) -> Self {
        Self::Account(account_event)
    }
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
