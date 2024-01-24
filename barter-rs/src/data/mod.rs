use serde::{Deserialize, Serialize};

/// Barter data module specific errors.
pub mod error;

/// Live market event feed for dry-trading & live-trading.
pub mod live;

/// Historical market event feed for backtesting.
pub mod historical;

/// Generates the next `Event`. Acts as the system heartbeat.
pub trait MarketGenerator<Event> {
    /// Return the next market `Event`.
    fn next(&mut self) -> Feed<Event>;
}

/// Communicates the state of the [`Feed`] as well as the next event.
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum Feed<Event> {
    Next(Event),
    Unhealthy,
    Finished,
}
