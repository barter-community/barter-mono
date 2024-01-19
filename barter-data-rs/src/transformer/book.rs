use crate::{
    error::DataError,
    event::{MarketEvent, MarketIter},
    exchange::Connector,
    subscription::{book::OrderBook, Map, SubKind},
    transformer::ExchangeTransformer,
    Identifier,
};
use async_trait::async_trait;
use barter_integration::{
    model::{instrument::Instrument, SubscriptionId},
    protocol::flat_files::BacktestMode,
    Transformer,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, marker::PhantomData, sync::Arc};

/// Defines how to apply a [`Self::Update`] to an [`Self::OrderBook`].
#[async_trait]
pub trait OrderBookUpdater
where
    Self: Sized,
{
    type OrderBook;
    type Update;
    type Snapshot;

    /// This often am HTTP call to receive a starting [`OrderBook`] snapshot.
    async fn get_snapshot<Exchange, Kind>(_: &Instrument) -> Result<Self::Snapshot, DataError>
    where
        Exchange: Send,
        Kind: Send;

    /// Initialises the [`InstrumentOrderBook`] for the provided [`Instrument`]
    fn init<Exchange, Kind>(
        _: Instrument,
        _: Self::Snapshot,
    ) -> Result<InstrumentOrderBook<Self>, DataError>
    where
        Exchange: Send,
        Kind: Send;

    /// Apply the [`Self::Update`] to the provided mutable [`Self::OrderBook`].
    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::Update,
    ) -> Result<Option<Self::OrderBook>, DataError>;
}

/// [`OrderBook`] for an [`Instrument`] with an exchange specific [`OrderBookUpdater`] to define
/// how to update it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InstrumentOrderBook<Updater> {
    pub instrument: Instrument,
    pub updater: Updater,
    pub book: OrderBook,
}

/// Standard generic [`ExchangeTransformer`] to translate exchange specific OrderBook types into
/// normalised Barter OrderBook types. Requires an exchange specific [`OrderBookUpdater`]
/// implementation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MultiBookTransformer<Exchange, Kind, Updater> {
    pub book_map: Map<InstrumentOrderBook<Updater>>,
    phantom: PhantomData<(Exchange, Kind)>,
}

impl<Exchange, Kind, Updater> MultiBookTransformer<Exchange, Kind, Updater> {
    pub fn init_book_map(book_map: Map<InstrumentOrderBook<Updater>>) -> Result<Self, DataError> {
        // make sure to sort the orderbooks
        for (_, inst_book) in book_map.0.iter() {
            let mut lock = inst_book.book.book.lock();
            lock.asks.sort();
            lock.bids.sort();
        }

        Ok(Self {
            book_map,
            phantom: PhantomData::default(),
        })
    }
}

#[async_trait]
impl<Exchange, Kind, Updater> ExchangeTransformer<Exchange, Kind>
    for MultiBookTransformer<Exchange, Kind, Updater>
where
    Exchange: Connector + Send,
    Kind: SubKind<Event = OrderBook> + Send,
    Updater: OrderBookUpdater<OrderBook = Kind::Event> + Send + Clone,
    Updater::Update: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
    Updater::Snapshot: Serialize,
{
    async fn new(_map: Map<Instrument>, _backtest_mode: BacktestMode) -> Result<Self, DataError> {
        // Construct empty OrderBookMap
        let book_map = HashMap::new();

        Ok(Self {
            book_map: Map(book_map),
            phantom: PhantomData::default(),
        })
    }

    async fn init_connection(
        &mut self,
        map: Map<Instrument>,
        backtest_mode: BacktestMode,
    ) -> Result<&Self, DataError> {
        // Initialise InstrumentOrderBooks for all Subscriptions
        let (sub_ids, init_book_requests): (Vec<_>, Vec<_>) = map
            .0
            .into_iter()
            .map(|(sub_id, instrument)| {
                let sub_id = Arc::new(sub_id);
                let sub_id_clone = sub_id.clone();
                let order_book = || async move {
                    let snapshot = Updater::get_snapshot::<Exchange, Kind>(&instrument).await?;

                    if backtest_mode == BacktestMode::ToFile {
                        let time = chrono::Local::now().format("%Y-%m-%d").to_string();
                        let file_name = format!("data/snapshot_{}_{}.json", &sub_id_clone, time);
                        // Serialize the map to a JSON string
                        let serialized = serde_json::to_string(&snapshot).unwrap();

                        // Create a file to write to
                        let mut file = File::create(file_name).unwrap();

                        // Write the JSON string to the file, handling any errors
                        file.write_all(serialized.as_bytes()).unwrap();
                    }

                    Updater::init::<Exchange, Kind>(instrument, snapshot)
                };

                (sub_id, order_book())
            })
            .unzip();

        // Await all initial OrderBook snapshot requests
        let init_order_books = futures::future::join_all(init_book_requests)
            .await
            .into_iter()
            .collect::<Result<Vec<InstrumentOrderBook<Updater>>, DataError>>()?;

        // Construct OrderBookMap if all requests successful
        let book_map = sub_ids
            .into_iter()
            .map(|sub_id| Arc::<SubscriptionId>::try_unwrap(sub_id).unwrap())
            .zip(init_order_books.into_iter())
            .collect::<Map<InstrumentOrderBook<Updater>>>();

        // make sure to sort the orderbooks
        for (_, inst_book) in book_map.0.iter() {
            let mut lock = inst_book.book.book.lock();
            lock.asks.sort();
            lock.bids.sort();
        }

        self.book_map = book_map;

        Ok(self)
    }
}

impl<Exchange, Kind, Updater> Transformer for MultiBookTransformer<Exchange, Kind, Updater>
where
    Exchange: Connector,
    Kind: SubKind<Event = OrderBook>,
    Updater: OrderBookUpdater<OrderBook = Kind::Event>,
    Updater::Update: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
{
    type Error = DataError;
    type Input = Updater::Update;
    type Output = MarketEvent<Kind::Event>;
    type OutputIter = Vec<Result<Self::Output, Self::Error>>;

    fn transform(&mut self, update: Self::Input) -> Self::OutputIter {
        // Determine if the update has an identifiable SubscriptionId
        let subscription_id = match update.id() {
            Some(subscription_id) => subscription_id,
            None => return vec![],
        };

        // Retrieve the InstrumentOrderBook associated with this update (snapshot or delta)
        let book = match self.book_map.find_mut(&subscription_id) {
            Ok(book) => book,
            Err(unidentifiable) => return vec![Err(DataError::Socket(unidentifiable))],
        };

        // De-structure for ease
        let InstrumentOrderBook {
            instrument,
            book,
            updater,
        } = book;

        let mut book = book.clone();

        // Apply update (snapshot or delta) to OrderBook & generate Market<OrderBook> snapshot
        match updater.update(&mut book, update) {
            Ok(Some(book)) => {
                MarketIter::<OrderBook>::from((Exchange::ID, instrument.clone(), book)).0
            }
            Ok(None) => vec![],
            Err(error) => vec![Err(error)],
        }
    }
}
