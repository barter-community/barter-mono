use crate::{
    event::Event,
    portfolio::{error::PortfolioError, position::PositionUpdate},
    strategy::{Signal, SignalForceExit},
};
use barter_data::event::{DataKind, MarketEvent};
use barter_execution::{fill::FillEvent, model::order_event::OrderEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Logic for [`OrderEvent`] quantity allocation.
pub mod allocator;

/// Barter portfolio module specific errors.
pub mod error;

/// Core Portfolio logic containing an implementation of [`MarketUpdater`],
/// [`OrderGenerator`] and [`FillUpdater`]. Utilises the risk and allocator logic to optimise
/// [`OrderEvent`] generation.
pub mod portfolio;

/// Data structures encapsulating the state of a trading [`Position`](position::Position), as
/// well as the logic for entering, updating and exiting them.
pub mod position;

/// Repositories for persisting Portfolio state.
pub mod repository;

/// Logic for evaluating the risk associated with a proposed [`OrderEvent`].
pub mod risk;

/// Updates the Portfolio from an input [`MarketEvent`].
pub trait MarketUpdater {
    /// Determines if the Portfolio has an open Position relating to the input [`MarketEvent`]. If
    /// so it updates it using the market data, and returns a [`PositionUpdate`] detailing the
    /// changes.
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError>;
}

/// May generate an [`OrderEvent`] from an input advisory [`Signal`].
pub trait OrderGenerator {
    /// May generate an [`OrderEvent`] after analysing an input advisory [`Signal`].
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError>;

    /// Generates an exit [`OrderEvent`] if there is an open [`Position`](position::Position)
    /// associated with the input [`SignalForceExit`]'s [`PositionId`](position::PositionId).
    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError>;
}

/// Updates the Portfolio from an input [`FillEvent`].
pub trait FillUpdater {
    /// Updates the Portfolio state using the input [`FillEvent`]. The [`FillEvent`] triggers a
    /// Position entry or exit, and the Portfolio updates key fields such as current_cash and
    /// current_value accordingly.
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError>;
}

/// Communicates a String represents a unique identifier for an Engine's Portfolio [`Balance`].
pub type BalanceId = String;

/// Total and available balance at a point in time.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Balance {
    pub time: DateTime<Utc>,
    pub total: f64,
    pub available: f64,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            total: 0.0,
            available: 0.0,
        }
    }
}

impl Balance {
    /// Construct a new [`Balance`] using the provided total & available balance values.
    pub fn new(time: DateTime<Utc>, total: f64, available: f64) -> Self {
        Self {
            time,
            total,
            available,
        }
    }

    /// Returns the unique identifier for an Engine's [`Balance`].
    pub fn balance_id(engine_id: Uuid) -> BalanceId {
        format!("{}_balance", engine_id)
    }
}
