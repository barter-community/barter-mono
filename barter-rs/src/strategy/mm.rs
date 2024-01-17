use super::glft::*;
use super::{Signal, SignalGenerator};
use barter_data::{
    event::{DataKind, MarketEvent},
    subscription::trade::PublicTrade,
};
use chrono::{DateTime, Timelike, Utc};
use ndarray::prelude::*;
use serde::{Deserialize, Serialize};

use tracing::error;

const INTERVAL: i64 = 100; // ms interval for measurments
const GAMMA: f64 = 0.05;
const DELTA: f64 = 0.1;

/// Configuration for constructing a [`RSIStrategy`] via the new() constructor method.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Config {}

#[derive(Clone, Debug)]
/// GLFTStrategy based strategy that implements [`SignalGenerator`].
pub struct GLFTStrategy {
    pub last_trades: Vec<PublicTrade>,
    pub last_exchange_time: Option<DateTime<Utc>>,
    pub measurement_params: MeasurementParams,
    pub arrival_depth: Array<f64, Ix1>,
    pub mid_price_chg: Array<f64, Ix1>,
    c1: f64,
    c2: f64,
    volatility: f64,
    last_update: DateTime<Utc>,
}

impl SignalGenerator for GLFTStrategy {
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal> {
        // Check if it's a MarketEvent with a candle
        let _trade = match &market.kind {
            DataKind::Trade(trade) => {
                // Run measurements
                self.run_measurements(trade, market);
                Some(trade)
            }
            _ => None,
        };

        // Check if it's a MarketEvent with a candle
        let _book = match &market.kind {
            DataKind::OrderBook(book) => {
                let m = &mut self.measurement_params;
                match (book.best_bid(), book.best_ask()) {
                    (Some(best_bid), Some(best_ask)) => {
                        m.best_bid_tick = best_bid / m.tick_size;
                        m.best_ask_tick = best_ask / m.tick_size;
                    }
                    _ => (),
                }
                Some(book)
            }
            _ => return None,
        };

        return None;

        // Generate advisory signals map
        // let signals = GLFTStrategy::do_stuff(trade);

        // // If signals map is empty, return no SignalEvent
        // if signals.is_empty() {
        //     return None;
        // }

        // Some(Signal {
        //     time: Utc::now(),
        //     exchange: market.exchange.clone(),
        //     instrument: market.instrument.clone(),
        //     market_meta: MarketMeta {
        //         close: trade.price,
        //         time: market.exchange_time,
        //     },
        //     signals,
        // })
    }

    fn on_market_feed_finished(&mut self) {
        self.update_strategy_params();
    }
}

impl GLFTStrategy {
    /// Constructs a new [`RSIStrategy`] component using the provided configuration struct.
    pub fn new(_: Config) -> Self {
        Self {
            last_trades: Vec::new(),
            last_exchange_time: None,
            measurement_params: MeasurementParams::default(),
            arrival_depth: Array::from_elem(10_000_000, f64::NAN),
            mid_price_chg: Array::from_elem(10_000_000, f64::NAN),
            c1: f64::NAN,
            c2: f64::NAN,
            volatility: f64::NAN,
            last_update: Utc::now(),
        }
    }

    fn run_measurements(&mut self, trade: &PublicTrade, market: &MarketEvent<DataKind>) {
        let event_time = market.exchange_time;
        if let None = self.last_exchange_time {
            self.last_exchange_time = Some(event_time);
            self.last_update = event_time;
        }
        let elapsed = event_time - self.last_exchange_time.unwrap();
        self.last_trades.push(trade.clone());

        // TODO we actually want to run this at fixed 100ms intervals
        if elapsed.num_milliseconds() >= INTERVAL {
            // this updates measurement_params
            measure_trading_intensity_and_volatility(
                &mut self.last_trades,
                &mut self.measurement_params,
                self.arrival_depth.view_mut(),
                self.mid_price_chg.view_mut(),
            );
            self.last_exchange_time = Some(event_time);
            self.last_trades.clear();

            // just in case we run out of space in the arrays

            let index = self.measurement_params.index;
            if index == self.arrival_depth.len() - 1 {
                reset_array(&mut self.mid_price_chg, index);
                reset_array(&mut self.arrival_depth, index);
                self.measurement_params.index = 6000;
            }

            // --------------------------------------------------------
            // Calibrates A, k and calculates the market volatility.

            let elapsed = event_time - self.last_update;

            // Updates A, k, and the volatility every 5-sec. (initiali window is 10 min)
            if elapsed.num_seconds() > 5 && self.measurement_params.index > 6000 {
                self.update_strategy_params();
                self.last_update = event_time;
            }
        }
    }

    #[allow(non_snake_case)]
    fn update_strategy_params(&mut self) {
        match get_params(
            self.arrival_depth.view(),
            self.mid_price_chg.view(),
            self.measurement_params.index,
        ) {
            Ok((A, k, volatility)) => {
                // println!("A: {}", A);
                // println!("k: {}", k);
                let (c1, c2) = compute_coeff(GAMMA, GAMMA, DELTA, A, k);
                self.c1 = c1;
                self.c2 = c2;
                self.volatility = volatility;
            }
            err => {
                error!("Failed to calibrate model: {:?}", err);
                return;
            }
        }
        let half_spread = 1.0 * self.c1 + 1.0 / 2.0 * self.c2 * self.volatility;
        let skew = self.c2 * self.volatility;

        if (self.last_update.minute() % 10) == 0 && self.last_update.second() < 5 {
            println!("~~~~~~");
            println!("date: {}", self.last_update);
            println!("c1: {}", self.c1);
            println!("c2: {}", self.c2);
            println!("volatility: {}", self.volatility);
            println!("half_spread: {}", half_spread);
            println!("skew: {}", skew);
        }
    }

    // /// Given the latest RSI value for a symbol, generates a map containing the [`SignalStrength`] for
    // /// [`Decision`] under consideration.
    // fn do_stuff(trade: &PublicTrade) -> HashMap<Decision, SignalStrength> {
    //     let mut signals = HashMap::with_capacity(4);
    //     let rsi = 0.0;
    //     if rsi < 40.0 {
    //         signals.insert(Decision::Long, GLFTStrategy::calculate_signal_strength());
    //     }
    //     if rsi > 60.0 {
    //         signals.insert(
    //             Decision::CloseLong,
    //             GLFTStrategy::calculate_signal_strength(),
    //         );
    //     }
    //     if rsi > 60.0 {
    //         signals.insert(Decision::Short, GLFTStrategy::calculate_signal_strength());
    //     }
    //     if rsi < 40.0 {
    //         signals.insert(
    //             Decision::CloseShort,
    //             GLFTStrategy::calculate_signal_strength(),
    //         );
    //     }
    //     signals
    // }

    // /// Calculates the [`SignalStrength`] of a particular [`Decision`].
    // fn calculate_signal_strength() -> SignalStrength {
    //     SignalStrength(1.0)
    // }
}
