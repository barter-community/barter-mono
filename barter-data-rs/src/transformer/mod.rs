use crate::{
    error::DataError,
    event::MarketEvent,
    subscription::{Map, SubKind},
};
use async_trait::async_trait;
use barter_integration::{
    model::instrument::Instrument,
    protocol::{flat_files::BacktestMode, websocket::WsMessage},
    Transformer,
};
use tokio::sync::mpsc;

/// Generic OrderBook [`ExchangeTransformer`]s.
pub mod book;

/// Generic stateless [`ExchangeTransformer`] often used for transforming
/// [`PublicTrades`](crate::subscription::trade::PublicTrades) streams.
pub mod stateless;

/// Defines how to construct a [`Transformer`] used by [`MarketStream`](super::MarketStream)s to
/// translate exchange specific types to normalised Barter types.
#[async_trait]
pub trait ExchangeTransformer<Exchange, Kind>
where
    Self: Transformer<Output = MarketEvent<Kind::Event>, Error = DataError> + Sized + Clone,
    Kind: SubKind,
{
    /// Construct a new [`Self`].
    ///
    async fn new(
        instrument_map: Map<Instrument>,
        backtest_mode: BacktestMode,
    ) -> Result<Self, DataError>;

    async fn init_connection(
        &mut self,
        _instrument_map: Map<Instrument>,
        _backtest_mode: BacktestMode,
    ) -> Result<&Self, DataError> {
        Ok(self)
    }

    /// The [`mpsc::UnboundedSender`] can be used by [`Self`] to send messages back to the exchange.
    async fn add_sender(
        &mut self,
        _ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    ) -> Result<(), DataError> {
        Ok(())
    }
}
