use barter::{
    data::historical,
    engine::{trader::Trader, Engine},
    event::{Event, EventTx},
    execution::{
        simulated::{Config as ExecutionConfig, SimulatedExecution},
        Fees,
    },
    portfolio::{
        allocator::DefaultAllocator, portfolio::MetaPortfolio,
        repository::in_memory::InMemoryRepository, risk::DefaultRisk,
    },
    statistic::summary::{
        trading::{Config as StatisticConfig, TradingSummary},
        Initialiser,
    },
    strategy::example::{Config as StrategyConfig, RSIStrategy},
};
use barter_data::{
    event::{DataKind, MarketEvent},
    exchange::{
        binance::{
            book::l2::BinanceOrderBookL2Snapshot,
            spot::{
                l2::{BinanceSpotBookUpdater, BinanceSpotOrderBookL2Delta},
                BinanceSpot,
            },
        },
        Connector, ExchangeId,
    },
    subscriber::mapper::WebSocketSubMapper,
    subscription::book::{OrderBook, OrderBooksL2},
    subscription::Map,
    transformer::book::{InstrumentOrderBook, MultiBookTransformer, OrderBookUpdater},
};
use barter_integration::{
    model::{
        instrument::{self, kind::InstrumentKind, Instrument},
        Exchange, Market, SubscriptionId,
    },
    protocol::{
        websocket::{WebSocketParser, WsMessage},
        StreamParser,
    },
    Transformer,
};
use chrono::Utc;
use parking_lot::Mutex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fs, sync::Arc};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::{fs::File, pin};
// use tokio_stream::{self as stream, StreamExt};
use futures::stream::{self, StreamExt};

use uuid::Uuid;

const ORDER_BOOK_DELTAS: &str = "data/binance_l2_2024_01_10_19.dat";
const SNAPSHOT: &str = "data/snapshot_@depth@100ms|ETHUSDT_2024-01-10.json";

#[tokio::main]
async fn main() -> io::Result<()> {
    let snapshot: BinanceOrderBookL2Snapshot = load_snapshot::<BinanceSpotBookUpdater>(SNAPSHOT);
    let updater = BinanceSpotBookUpdater::new(snapshot.last_update_id);

    let book = OrderBook::from(snapshot);

    let instrument = Instrument::from(("eth", "usdt", InstrumentKind::Spot));

    let mut transformer = build_transformer::<BinanceSpot, OrderBooksL2, BinanceSpotBookUpdater>(
        instrument, book, updater,
    )
    .unwrap();

    // let line = "{\"e\":\"depthUpdate\",\"E\":1704479692481,\"s\":\"ETHUSDT\",\"U\":28651857125,\"u\":28651857130,\"b\":[[\"2229.76000000\",\"0.07030000\"],[\"2228.87000000\",\"16.70590000\"],[\"2226.15000000\",\"0.00000000\"]],\"a\":[[\"2230.01000000\",\"81.20450000\"],[\"2230.20000000\",\"32.71050000\"],[\"2285.58000000\",\"0.00000000\"]]}".to_string();
    let mut lines = Box::pin(line_stream(ORDER_BOOK_DELTAS).await.unwrap());

    while let Some(line) = lines.next().await {
        // println!("{}", &line);
        let msg = WsMessage::Text(line.unwrap());

        // Parse input protocol message into `ExchangeMessage`
        let exchange_message: BinanceSpotOrderBookL2Delta = match WebSocketParser::parse(Ok(msg)) {
            // `StreamParser` successfully deserialised `ExchangeMessage`
            Some(Ok(exchange_message)) => exchange_message,

            // If `StreamParser` returns an Err pass it downstream
            Some(Err(err)) => panic!("{:?}", err),

            // If `StreamParser` returns None it's a safe-to-skip message
            None => panic!("failed to parse2"),
        };
        // println!("exchange_message {:?}", exchange_message);

        if "@depth@100ms|ETHUSDT" != exchange_message.subscription_id.0 {
            continue;
        }

        transformer
            .transform(exchange_message)
            .into_iter()
            .for_each(|event| {
                println!("{:?}", event);
                println!("--");
            });
    }
    Ok(())
}

// #[tokio::main]
// async fn main() {
//     // Create channel to distribute Commands to the Engine & it's Traders (eg/ Command::Terminate)
//     // let (_command_tx, command_rx) = mpsc::channel(20);

//     // Create Event channel to listen to all Engine Events in real-time
//     let (event_tx, event_rx) = mpsc::unbounded_channel();
//     let event_tx = EventTx::new(event_tx);

//     // Generate unique identifier to associate an Engine's components
//     let engine_id = Uuid::new_v4();

//     // Create the Market(s) to be traded on (1-to-1 relationship with a Trader)
//     let market = Market::new("binance", ("btc", "usdt", InstrumentKind::Spot));

//     // Build global shared-state MetaPortfolio (1-to-1 relationship with an Engine)
//     let portfolio = Arc::new(Mutex::new(
//         MetaPortfolio::builder()
//             .engine_id(engine_id)
//             .markets(vec![market.clone()])
//             .starting_cash(10_000.0)
//             .repository(InMemoryRepository::new())
//             .allocation_manager(DefaultAllocator {
//                 default_order_value: 100.0,
//             })
//             .risk_manager(DefaultRisk {})
//             .statistic_config(StatisticConfig {
//                 starting_equity: 10_000.0,
//                 trading_days_per_year: 365,
//                 risk_free_return: 0.0,
//             })
//             .build_and_init()
//             .expect("failed to build & initialise MetaPortfolio"),
//     ));

//     // Build Trader(s)
//     let mut traders = Vec::new();

//     // Create channel for each Trader so the Engine can distribute Commands to it
//     let (trader_command_tx, trader_command_rx) = mpsc::channel(10);

//     // return;

//     // traders.push(
//     //     Trader::builder()
//     //         .engine_id(engine_id)
//     //         .market(market.clone())
//     //         .command_rx(trader_command_rx)
//     //         .event_tx(event_tx.clone())
//     //         .portfolio(Arc::clone(&portfolio))
//     //         // .data(historical::MarketFeed::new(
//     //         //     // load_json_market_event_candles().into_iter(),
//     //         // ))
//     //         .strategy(RSIStrategy::new(StrategyConfig { rsi_period: 14 }))
//     //         .execution(SimulatedExecution::new(ExecutionConfig {
//     //             simulated_fees_pct: Fees {
//     //                 exchange: 0.1,
//     //                 slippage: 0.05,
//     //                 network: 0.0,
//     //             },
//     //         }))
//     //         .build()
//     //         .expect("failed to build trader"),
//     // );

//     // // Build Engine (1-to-many relationship with Traders)
//     // // Create HashMap<Market, trader_command_tx> so Engine can route Commands to Traders
//     // let trader_command_txs = HashMap::from([(market, trader_command_tx)]);

//     // let engine = Engine::builder()
//     //     .engine_id(engine_id)
//     //     .command_rx(command_rx)
//     //     .portfolio(portfolio)
//     //     .traders(traders)
//     //     .trader_command_txs(trader_command_txs)
//     //     .statistics_summary(TradingSummary::init(StatisticConfig {
//     //         starting_equity: 1000.0,
//     //         trading_days_per_year: 365,
//     //         risk_free_return: 0.0,
//     //     }))
//     //     .build()
//     //     .expect("failed to build engine");

//     // // Run Engine trading & listen to Events it produces
//     // tokio::spawn(listen_to_engine_events(event_rx));
//     // engine.run().await;
// }

async fn line_stream(
    file_path: &str,
) -> io::Result<impl stream::Stream<Item = io::Result<String>>> {
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let stream = stream::unfold(reader, |mut reader| async {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await;
        match bytes_read {
            Ok(0) => None, // EOF
            Ok(_) => Some((Ok(line), reader)),
            Err(e) => Some((Err(e), reader)),
        }
    });
    Ok(stream)
}

fn load_snapshot<Updater>(filename: &str) -> Updater::Snapshot
where
    Updater: OrderBookUpdater,
    Updater::Snapshot: for<'de> Deserialize<'de>,
{
    let snapshot = fs::read_to_string(filename).expect("failed to read file");

    let snapshot: Updater::Snapshot =
        serde_json::from_str(&snapshot).expect("failed to parse order book snapshot");

    snapshot
}

fn build_transformer<Exchange, Kind, Updater>(
    instrument: Instrument,
    book: OrderBook,
    updater: Updater,
) -> io::Result<MultiBookTransformer<Exchange, Kind, Updater>>
where
    Exchange: Connector,
{
    let book = InstrumentOrderBook {
        instrument,
        book,
        updater,
    };

    let ex_id = Exchange::ID.to_string();
    let id: SubscriptionId = SubscriptionId(String::from(ex_id));

    // TODO need to figure out how to get this
    let id = SubscriptionId("@depth@100ms|ETHUSDT".to_string());

    let mut book_map = HashMap::new();
    book_map.insert(id, book);

    let transformer = MultiBookTransformer::<Exchange, Kind, Updater>::init_book_map(Map(book_map));

    match transformer {
        Ok(transformer) => return Ok(transformer),
        Err(err) => panic!("{:?}", err),
    }
}

// Listen to Events that occur in the Engine. These can be used for updating event-sourcing,
// updating dashboard, etc etc.
async fn listen_to_engine_events(mut event_rx: mpsc::UnboundedReceiver<Event>) {
    while let Some(event) = event_rx.recv().await {
        match event {
            Event::Market(_) => {
                // Market Event occurred in Engine
            }
            Event::Signal(signal) => {
                // Signal Event occurred in Engine
                println!("{signal:?}");
            }
            Event::SignalForceExit(_) => {
                // SignalForceExit Event occurred in Engine
            }
            Event::OrderNew(new_order) => {
                // OrderNew Event occurred in Engine
                println!("{new_order:?}");
            }
            Event::OrderUpdate => {
                // OrderUpdate Event occurred in Engine
            }
            Event::Fill(fill_event) => {
                // Fill Event occurred in Engine
                println!("{fill_event:?}");
            }
            Event::PositionNew(new_position) => {
                // PositionNew Event occurred in Engine
                println!("{new_position:?}");
            }
            Event::PositionUpdate(updated_position) => {
                // PositionUpdate Event occurred in Engine
                println!("{updated_position:?}");
            }
            Event::PositionExit(exited_position) => {
                // PositionExit Event occurred in Engine
                println!("{exited_position:?}");
            }
            Event::Balance(balance_update) => {
                // Balance update Event occurred in Engine
                println!("{balance_update:?}");
            }
        }
    }
}
