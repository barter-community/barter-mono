use barter_data::{
    exchange::{
        binance::{
            spot::{l2::BinanceSpotBookUpdater, BinanceServerSpot, BinanceSpot},
            Binance,
        },
        Connector, ExchangeId,
    },
    streams::{builder::validate, consumer::consume_new, Streams},
    subscriber::{mapper::SubscriptionMapper, Subscriber},
    subscription::{book::OrderBooksL2, Subscription, SubscriptionMeta},
    transformer::{book::MultiBookTransformer, ExchangeTransformer},
};
use barter_integration::{
    model::instrument::kind::InstrumentKind, protocol::flat_files::BacktestMode,
};

use tracing::info;

static BACKTEST_MODE: BacktestMode = BacktestMode::None;

#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Hand-init

    let subs = Subscription::from((
        BinanceSpot::default(),
        "eth",
        "usdt",
        InstrumentKind::Spot,
        OrderBooksL2,
    ));
    let subs = [subs];
    let mut subscriptions = subs.into_iter().collect::<Vec<_>>();

    // Validate Subscriptions
    validate(&subscriptions).unwrap();

    // Remove duplicate Subscriptions
    subscriptions.sort();
    subscriptions.dedup();

    let mut stream_builder = Streams::<OrderBooksL2>::builder::<OrderBooksL2>();
    let exchange_tx = stream_builder.get_ex_tx(BinanceSpot::ID);

    let fut = Box::pin(async move {
        // TODO: refactor this
        let SubscriptionMeta {
            instrument_map,
            subscriptions: _subs,
        } = <<Binance<BinanceServerSpot> as Connector>::Subscriber as Subscriber>::SubMapper::map(
            &subscriptions,
        );

        let transformer: MultiBookTransformer<BinanceSpot, OrderBooksL2, BinanceSpotBookUpdater> =
            MultiBookTransformer::new(instrument_map, BACKTEST_MODE)
                .await
                .unwrap();
        tokio::spawn(consume_new::<BinanceSpot, OrderBooksL2>(
            subscriptions,
            exchange_tx,
            transformer,
            BACKTEST_MODE,
        ));
        Ok(())
    });

    let mut streams = stream_builder.add_fut(fut).init().await.unwrap();

    // Select the ExchangeId::BinanceSpot stream
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    let mut binance_stream = streams.select(ExchangeId::BinanceSpot).unwrap();

    // Spawn a new asynchronous task to handle writing to the file
    while let Some(order_book_l2) = binance_stream.recv().await {
        info!("{order_book_l2:?}");
    }
}

// fn build_transformer<Exchange, Kind, Updater>(
//     instrument: Instrument,
//     book: OrderBook,
//     updater: Updater,
// ) -> io::Result<MultiBookTransformer<Exchange, Kind, Updater>>
// where
//     Exchange: Connector,
// {
//     let book = InstrumentOrderBook {
//         instrument,
//         book,
//         updater,
//     };

//     let ex_id = Exchange::ID.to_string();
//     let id: SubscriptionId = SubscriptionId(String::from(ex_id));

//     // TODO need to figure out how to get this
//     let id = SubscriptionId("@depth@100ms|ETHUSDT".to_string());

//     let map;
//     let transformer = MultiBookTransformer::new(map, BACKTEST_MODE);

//     match transformer {
//         Ok(transformer) => return Ok(transformer),
//         Err(err) => panic!("{:?}", err),
//     }
// }

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
