use barter_integration::model::Exchange;

use super::order::{Order, RequestCancel, RequestOpen};

// Todo: If we pass tuple (Exchange, Order<Request>), the OrderRequest should maybe be diff that doesn't include Exchange
#[derive(Debug)]
pub enum ExecutionRequest {
    // Fetch Account State
    FetchBalances(Vec<Exchange>),
    FetchOrdersOpen(Vec<Exchange>),

    OpenOrders(Vec<(Exchange, Vec<Order<RequestOpen>>)>),

    CancelOrders(Vec<(Exchange, Vec<Order<RequestCancel>>)>),
    CancelOrdersAll(Vec<Exchange>),
}

// Todo: If we pass tuple (Exchange, Order<Request>), the OrderRequest should maybe be diff that doesn't include Exchange
#[derive(Debug)]
pub enum ExchangeRequest {
    // Fetch Account State
    FetchBalances,
    FetchOrdersOpen,

    // Open Orders
    OpenOrders(Vec<Order<RequestOpen>>),

    CancelOrders(Vec<Order<RequestCancel>>),
    CancelOrdersAll,
}
