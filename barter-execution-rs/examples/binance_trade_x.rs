use barter_execution::{
    execution::binance::{
        connection::BinanceApi,
        connection::{BinanceClient, LiveOrTest},
        requests::FutOrderResponse,
    },
    fill::{Decision, MarketMeta},
    model::order_event::{OrderEventBuilder, OrderExecutionType, OrderType},
};
use barter_integration::model::{
    instrument::{kind::InstrumentKind, Instrument},
    Exchange,
};
use chrono::Utc;

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    let order = OrderEventBuilder::new()
        .instrument(Instrument::new("ETH", "USDT", InstrumentKind::Perpetual))
        .decision(Decision::Long)
        .quantity(1.0)
        .order_type(OrderType::Market)
        // .order_type(OrderType::Limit {
        //     price: 1000.0,
        //     execution_type: OrderExecutionType::None,
        // })
        .time(Utc::now())
        .exchange(Exchange::from("binance"))
        .market_meta(MarketMeta::default())
        .build()
        .expect("Failed to build order");

    let rest_client = BinanceClient::new(BinanceApi::Futures(LiveOrTest::Test));

    // Build RestClient with Binance configuration
    match rest_client.submit_order::<FutOrderResponse>(&order).await {
        Ok(response) => println!("{:#?}", response),
        Err(e) => println!("{:?}", e),
    }
}
