use super::SubKind;
use barter_integration::model::Side;
use barter_macro::{DeSubKind, SerSubKind};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields [`OpenOrder`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DeSubKind, SerSubKind)]
pub struct OpenOrders;

impl SubKind for OpenOrders {
    type Event = OpenOrder;
}

/// Normalised Barter [`PublicTrade`] model.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OpenOrder {
    /// TODO
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
}
