use super::SubKind;
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields [`IntentOrder`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct IntentOrders;

impl SubKind for IntentOrders {
    type Event = IntentOrder;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum IntentOrderUpdate {
  Opened,
  Closed,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct IntentOrder {
    pub event: IntentOrderUpdate,
    pub id: String,
    pub in_token: String,
    pub in_amount: f64,
    pub out_token: String,
    pub out_amount: f64,
    pub start_ask: f64,
    pub end_ask: f64,
    pub price: f64,
    pub created_at: u64,
    pub order_type: String,
    pub signature: String,
    pub encoded_order: String,
}



