use crate::model::{order::OrderKind, ClientOrderId};
use barter_integration::{error::SocketError, model::instrument::symbol::Symbol};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Failed to build struct due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),

    #[error("SimulatedExchange error: {0}")]
    Simulated(String),

    #[error("Balance for symbol {0} insufficient to open order")]
    InsufficientBalance(Symbol),

    #[error("failed to find Order with ClientOrderId: {0}")]
    OrderNotFound(ClientOrderId),

    #[error("failed to open Order due to unsupported OrderKind: {0}")]
    UnsupportedOrderKind(OrderKind),

    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}
