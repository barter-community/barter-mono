use super::event::Event;
use barter_execution::model::order::{Order, RequestCancel, RequestOpen};
use barter_execution::model::AccountEvent;
use barter_execution::simulated::execution::SimulatedExecution;
use barter_execution::{
    execution::binance::BinanceConfig, simulated::execution::SimulationConfig, ExecutionClient,
};
use barter_integration::model::Exchange;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;

/// Responsibilities:
/// - Determines best way to action an [`ExchangeRequest`] given the constraints of the exchange.

/// Responsibilities:
/// - Manages every [`ExchangeClient`].
/// - Forwards an [`ExchangeRequest`] to the appropriate [`ExchangeClient`].
/// - Map InternalClientOrderId to exchange ClientOrderId.

#[derive(Debug)]
pub struct ExchangePortal<Client>
where
    Client: ExecutionClient,
{
    clients: HashMap<Exchange, mpsc::UnboundedReceiver<ExecutionRequest>>,
    request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl<Client> ExchangePortal<Client>
where
    Client: ExecutionClient,
{
    pub async fn init(
        exchanges: HashMap<Exchange, ClientId>,
        // event_tx: mpsc::UnboundedSender<Event>,
        event_tx: mpsc::UnboundedSender<AccountEvent>,
    ) -> Result<Self, ()> {
        // Todo:
        //  - Validate input
        //  - I don't think there is any reason the core would ask for ConnectionStatus, but it would be sent
        //  - Can ExchangePortal act as the Driver? Yes.
        //  - Make ExchangePortal state machine...

        // 1. Store HashMap<Exchange, ClientId> for association & to keep every ClientId(Config)
        // 2. Spawn tasks for every ExchangeClient
        // 3. Monitor ConnectionStatus of each task
        // 4. Re-spawn ExchangeClient if required

        let clients: HashMap<Exchange, Client> = HashMap::new();

        for (exchange, client_id) in exchanges {
            match client_id {
                ClientId::Simulated(config) => {
                    let client = SimulatedExecution::init(config, event_tx.clone()).await;
                    clients.insert(exchange.clone(), client);
                }
                ClientId::Binance(config) => {}
            }

            // Runner
        }

        Err(())
    }

    /// Todo:
    ///  - Should be run on it's own OS thread.
    ///  - This may live in Barter... ExchangeClient impls would live here. Order would be in Barter!
    ///  - Just use HTTP for trading for the time being...
    ///  - May need to run enum ExchangeEvent { request, ConnectionStatus } in order to re-spawn clients! -> state machine like Cerebrum!
    pub fn run(mut self) {
        loop {
            // Receive next ExchangeRequest
            let request = match self.request_rx.try_recv() {
                Ok(request) => request,
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => panic!("todo"),
            };
            info!(payload = ?request, "received ExchangeRequest");

            // Action ExecutionRequest
            match request {
                ExecutionRequest::FetchOrdersOpen(exchanges) => {}
                ExecutionRequest::FetchBalances(exchanges) => {}
                ExecutionRequest::OpenOrders(open_requests) => {}
                ExecutionRequest::CancelOrders(cancel_requests) => {}
                ExecutionRequest::CancelOrdersAll(exchanges) => {}
            }
        }
    }

    /// Retrieve the [`ExchangeClient`] associated with the [`Exchange`].
    pub fn client(&mut self, exchange: &Exchange) -> &Client {
        self.clients
            .get(exchange)
            .expect("cannot retrieve ExchangeClient for unexpected Exchange")
    }
}

// Todo: If we pass tuple (Exchange, Order<Request>), the OrderRequest should maybe be diff that doesn't include Exchange
#[derive(Debug)]
pub enum ExecutionRequest {
    // Check ExchangeClient status
    // ClientStatus(Vec<Exchange>),

    // Fetch Account State
    FetchBalances(Vec<Exchange>),
    FetchOrdersOpen(Vec<Exchange>),

    // Open Orders
    // OpenOrder(Order<RequestOpen>),
    // OpenOrderBatch(Order<Vec<RequestOpen>>),
    OpenOrders(Vec<(Exchange, Vec<Order<RequestOpen>>)>),

    // Cancel Orders
    // CancelOrderById,
    // CancelOrderByInstrument,
    // CancelOrderByBatch,
    CancelOrders(Vec<(Exchange, Vec<Order<RequestCancel>>)>),
    CancelOrdersAll(Vec<Exchange>),
}

#[derive(Debug, Clone, Copy)]
pub struct ClientHealth {
    status: ClientStatus,
    latency_avg: (),
}

#[derive(Clone, Copy, Debug)]
pub enum ClientStatus {
    Connected,
    CancelOnly,
    Disconnected,
}

// Todo:
//   - Better name for this? This is the equivilant to ExchangeId...
//    '--> renamed to ClientId for now to avoid confusion in development
#[derive(Debug)]
pub enum ClientId {
    Simulated(SimulationConfig),
    Binance(BinanceConfig),
}
