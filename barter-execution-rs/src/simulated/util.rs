use std::{collections::HashMap, time::Duration};

use barter_integration::model::instrument::{kind::InstrumentKind, symbol::Symbol, Instrument};
use tokio::sync::mpsc;

use crate::model::{balance::Balance, AccountEvent};

use super::{
    exchange::{
        account::{balance::ClientBalances, ClientAccount},
        SimulatedExchange,
    },
    SimulatedEvent,
};

pub async fn run_default_exchange(
    event_account_tx: mpsc::UnboundedSender<AccountEvent>,
    event_simulated_rx: mpsc::UnboundedReceiver<SimulatedEvent>,
) {
    // Define SimulatedExchange available Instruments
    let instruments = instruments();

    // Create initial ClientAccount balances (Symbols must all be included in the Instruments)
    let balances = initial_balances();

    // Build SimulatedExchange & run on it's own Tokio task
    SimulatedExchange::builder()
        .event_simulated_rx(event_simulated_rx)
        .account(
            ClientAccount::builder()
                .latency(latency_50ms())
                .fees_percent(fees_50_percent())
                .event_account_tx(event_account_tx)
                .instruments(instruments)
                .balances(balances)
                .build()
                .expect("failed to build ClientAccount"),
        )
        .build()
        .expect("failed to build SimulatedExchange")
        .run()
        .await
}

// Initial SimulatedExchange ClientAccount balances for each Symbol
pub fn initial_balances() -> ClientBalances {
    ClientBalances(HashMap::from([
        (Symbol::from("btc"), Balance::new(10.0, 10.0)),
        (Symbol::from("usdt"), Balance::new(10_000.0, 10_000.0)),
    ]))
}

// Instruments that the SimulatedExchange supports
pub fn instruments() -> Vec<Instrument> {
    vec![Instrument::from(("btc", "usdt", InstrumentKind::Perpetual))]
}

pub fn latency_50ms() -> Duration {
    Duration::from_millis(50)
}

pub fn fees_50_percent() -> f64 {
    0.5
}
