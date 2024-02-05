use super::{event::Event, exchange::ClientStatus};
use async_trait::async_trait;
use barter_execution::{
    error::ExecutionError,
    execution::binance::requests::FUT_BALANCES_REQUEST,
    model::{
        balance::SymbolBalance,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
    },
    ExecutionClient, ExecutionId,
};
use barter_integration::model::instrument::{symbol::Symbol, Instrument};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
