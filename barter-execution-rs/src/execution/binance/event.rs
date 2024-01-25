/// Events used to communicate with the Barter [`SimulatedExchange`](exchange::SimulatedExchange).
///
/// Two main types of [`SimulatedEvent`]:
/// 1. Request sent from the [`SimulatedExecution`](execution::SimulatedExecution)
///    [`ExecutionClient`](crate::ExecutionClient).
/// 2. Market events used to model available liquidity and trigger matches with open client orders.
#[derive(Debug)]
pub enum BinanceEvent {
    FetchOrdersOpen(oneshot::Sender<Result<Vec<Order<Open>>, ExecutionError>>),
    FetchBalances(oneshot::Sender<Result<Vec<SymbolBalance>, ExecutionError>>),
    OpenOrders(
        (
            Vec<Order<RequestOpen>>,
            oneshot::Sender<Vec<Result<Order<Open>, ExecutionError>>>,
        ),
    ),
    CancelOrders(
        (
            Vec<Order<RequestCancel>>,
            oneshot::Sender<Vec<Result<Order<Cancelled>, ExecutionError>>>,
        ),
    ),
    CancelOrdersAll(oneshot::Sender<Result<Vec<Order<Cancelled>>, ExecutionError>>),
    MarketTrade((Instrument, PublicTrade)),
}
