use super::ExchangeTransformer;
use crate::{
    event::{Market, MarketIter},
    exchange::{Connector, ExchangeId},
    subscription::{SubKind, SubscriptionMap},
    Identifier,
};
use barter_integration::{
    error::SocketError,
    model::{Instrument, SubscriptionId},
    protocol::websocket::WsMessage,
    Transformer,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tokio::sync::mpsc;

#[derive(Clone, Eq, PartialEq, Debug, Serialize)]
pub struct BookTransformer<Exchange, Kind, Input> {
    pub map: SubscriptionMap<Exchange, Kind>,
    phantom: PhantomData<Input>,
}

impl<Exchange, Kind, Input> ExchangeTransformer<Exchange, Kind>
    for BookTransformer<Exchange, Kind, Input>
where
    Exchange: Connector,
    Kind: SubKind,
    Input: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
    MarketIter<Kind::Event>: From<(ExchangeId, Instrument, Input)>,
{
    fn new(_: mpsc::UnboundedSender<WsMessage>, map: SubscriptionMap<Exchange, Kind>) -> Self {
        Self {
            map,
            phantom: Default::default(),
        }
    }
}

impl<Exchange, Kind, Input> Transformer for BookTransformer<Exchange, Kind, Input>
where
    Exchange: Connector,
    Kind: SubKind,
    Input: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
    MarketIter<Kind::Event>: From<(ExchangeId, Instrument, Input)>,
{
    type Input = Input;
    type Output = Market<Kind::Event>;
    type OutputIter = Vec<Result<Self::Output, SocketError>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Determine if the message has an identifiable SubscriptionId
        let subscription_id = match input.id() {
            Some(subscription_id) => subscription_id,
            None => return vec![],
        };

        todo!()
    }
}