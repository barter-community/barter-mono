use barter_data::{
    event::{DataKind, MarketEvent},
    exchange::{binance::spot::BinanceSpot, ExchangeId},
    streams::Streams,
    subscription::{book::OrderBooksL2, trade::PublicTrades},
};
use barter_integration::{
    model::instrument::kind::InstrumentKind, protocol::flat_files::BacktestMode,
};
use tracing::info;

// use tracing::info;

static BACKTEST_MODE: BacktestMode = BacktestMode::ToFile;

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    let trade_stream = Streams::<PublicTrades>::builder()
        .subscribe_bt([
            (BinanceSpot::default(), "eth", "usdt", InstrumentKind::Spot, PublicTrades),
        ], BACKTEST_MODE)
        .init()
        .await
        .unwrap();
  


    // Initialise OrderBooksL2 Streams for BinanceSpot only
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let mut order_book_streams = Streams::<OrderBooksL2>::builder()

        // Separate WebSocket connection for ETH_USDT stream since it's very high volume
        .subscribe_bt([
            (BinanceSpot::default(), "eth", "usdt", InstrumentKind::Spot, OrderBooksL2),
        ], BACKTEST_MODE)
        .init()
        .await
        .unwrap();

        // Separate WebSocket connection for BTC_USDT stream since it's very high volume
        // .subscribe([
        //     (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, OrderBooksL2),
        // ], BACKTEST_MODE)

        // Lower volume Instruments can share a WebSocket connection
        // .subscribe([
        //     (BinanceSpot::default(), "xrp", "usdt", InstrumentKind::Spot, OrderBooksL2),
        //     (BinanceSpot::default(), "sol", "usdt", InstrumentKind::Spot, OrderBooksL2),
        //     (BinanceSpot::default(), "avax", "usdt", InstrumentKind::Spot, OrderBooksL2),
        //     (BinanceSpot::default(), "ltc", "usdt", InstrumentKind::Spot, OrderBooksL2),
        // ], BACKTEST_MODE)

  ;


        // let streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()
        // .add(trade_stream)
        // .add(order_book_streams)
        // .init()
        // .await
        // .unwrap();
        // let mut joined_stream = streams.join_map().await;


    // Select the ExchangeId::BinanceSpot stream
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    let mut binance_stream = order_book_streams
        .select(ExchangeId::BinanceSpot)
        .unwrap();


    // Spawn a new asynchronous task to handle writing to the file
    while let Some( data) = binance_stream.recv().await {
        // info!("{data:#?}");
    }  
}

// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}
