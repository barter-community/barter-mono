use barter_data::{
    event::MarketEvent,
    exchange::{binance::spot::BinanceSpot, ExchangeId},
    streams::Streams,
    subscription::book::{OrderBook, OrderBooksL2},
};
use barter_integration::model::instrument::kind::InstrumentKind;
use chrono::Timelike;
use csv_async::AsyncSerializer;

use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::info;

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Initialise OrderBooksL2 Streams for BinanceSpot only
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let mut streams = Streams::<OrderBooksL2>::builder()

        // Separate WebSocket connection for BTC_USDT stream since it's very high volume
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, OrderBooksL2),
        ])

        // Separate WebSocket connection for ETH_USDT stream since it's very high volume
        .subscribe([
            (BinanceSpot::default(), "eth", "usdt", InstrumentKind::Spot, OrderBooksL2),
        ])

        // Lower volume Instruments can share a WebSocket connection
        .subscribe([
            (BinanceSpot::default(), "xrp", "usdt", InstrumentKind::Spot, OrderBooksL2),
            (BinanceSpot::default(), "sol", "usdt", InstrumentKind::Spot, OrderBooksL2),
            (BinanceSpot::default(), "avax", "usdt", InstrumentKind::Spot, OrderBooksL2),
            (BinanceSpot::default(), "ltc", "usdt", InstrumentKind::Spot, OrderBooksL2),
        ])
        .init()
        .await
        .unwrap();

    // Select the ExchangeId::BinanceSpot stream
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    let mut binance_stream = streams
        .select(ExchangeId::BinanceSpot)
        .unwrap();


    // Spawn a new asynchronous task to handle writing to the file
    while let Some(order_book_l2) = binance_stream.recv().await {
        // info!("MarketEvent<OrderBook>: {order_book_l2:?}");
        let rec_time = order_book_l2.received_time;
        let minute = (rec_time.minute() as f32 / 5.0).floor() as i32 * 5;
        let formatted = rec_time.format("%Y_%m_%d_%H:").to_string() + minute.to_string().as_str();
        let file_name = format!("data/binance_l2_{}.csv", formatted);
        write_mpsc_stream_to_file(order_book_l2, &file_name).await.unwrap();        
    }  
}

async fn write_mpsc_stream_to_file(
    order_book_l2: MarketEvent<OrderBook>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file in append mode using tokio's OpenOptions
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(output_path)
        .await?;

    // Create a CSV writer
    // let writer = BufWriter::new(file);

    // Create an asynchronous CSV serializer
    let mut csv_writer = AsyncSerializer::from_writer(file);

    // Receive messages from the channel and write them to the file
    csv_writer.serialize(order_book_l2).await?;

    // Flush any remaining output to the file
    csv_writer.flush().await?;

    Ok(())
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
