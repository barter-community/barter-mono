use super::event::Event;
use barter_execution::execution::binance::BinanceExecution;
use barter_execution::model::execution_event::{ExchangeRequest, ExecutionRequest};
use barter_execution::simulated::execution::SimulatedExecution;
use barter_execution::ExecutionId;
use barter_execution::{
    execution::binance::BinanceConfig, simulated::execution::SimulationConfig, ExecutionClient,
};
use barter_integration::model::Exchange;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::info;

/// Responsibilities:
/// - Determines best way to action an [`ExchangeRequest`] given the constraints of the exchange.

/// Responsibilities:
/// - Manages every [`ExchangeClient`].
/// - Forwards an [`ExchangeRequest`] to the appropriate [`ExchangeClient`].
/// - Map InternalClientOrderId to exchange ClientOrderId.

#[derive(Debug)]
pub struct ExchangePortal {
    // TODO: do we need to store exchanges? For re-initialization?
    exchanges: HashMap<ExecutionId, ClientId>,
    clients: HashMap<Exchange, mpsc::UnboundedSender<ExchangeRequest>>,
    request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl ExchangePortal {
    pub async fn init(
        exchanges: HashMap<ExecutionId, ClientId>,
        request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
        event_tx: mpsc::UnboundedSender<Event>,
    ) -> Result<Self, ()> {
        // Todo:
        //  - Validate input
        //  - I don't think there is any reason the core would ask for ConnectionStatus, but it would be sent
        //  - Can ExchangePortal act as the Driver? Yes.
        //  - Make ExchangePortal state machine...

        // 1. Store HashMap<Exchange, ClientId> for association & to keep every ClientId(Config)√
        // 2. Spawn tasks for every ExchangeClient √
        // 3. Monitor ConnectionStatus of each task √
        // 4. Re-spawn ExchangeClient if required TODO?

        let mut clients = HashMap::new();

        info!("initializing ExchangePortal {:?}", exchanges);

        for (execution_id, client_id) in exchanges.iter() {
            match client_id {
                ClientId::Simulated(config) => {
                    let (execution_tx, execution_rx) = mpsc::unbounded_channel();
                    let client = SimulatedExecution::init(config.clone(), event_tx.clone()).await;
                    clients.insert(Exchange::from(execution_id.clone()), execution_tx);
                    tokio::task::spawn(async move {
                        client.run(execution_rx).await;
                    });
                }
                ClientId::Binance(config) => {
                    let (execution_tx, execution_rx) = mpsc::unbounded_channel();
                    let client = BinanceExecution::init(config.clone(), event_tx.clone()).await;
                    clients.insert(Exchange::from(execution_id.clone()), execution_tx);
                    tokio::spawn(async move {
                        client.run(execution_rx).await;
                    });
                }
            }
        }

        Ok(Self {
            exchanges,
            clients,
            request_rx,
            event_tx,
        })
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
            // while let Some(request) = self.request_rx.recv().await {
            // info!(payload = ?request, "received ExchangeRequest");

            // Action ExecutionRequest
            match request {
                ExecutionRequest::FetchOrdersOpen(exchanges) => {
                    exchanges.into_iter().for_each(|exchange| {
                        let client = self.client(&exchange);
                        (*client)
                            .send(ExchangeRequest::FetchOrdersOpen)
                            .expect("failed to send FetchOrdersOpen to ExchangeClient");
                    });
                }
                ExecutionRequest::FetchBalances(exchanges) => {
                    exchanges.into_iter().for_each(|exchange| {
                        let client = self.client(&exchange);
                        (*client)
                            .send(ExchangeRequest::FetchBalances)
                            .expect("failed to send FetchBalances to ExchangeClient");
                    });
                }
                ExecutionRequest::OpenOrders(open_requests) => {
                    open_requests.into_iter().for_each(|open_request| {
                        let client = self.client(&open_request.0);
                        info!("sending OpenOrders ");
                        (*client)
                            .send(ExchangeRequest::OpenOrders(open_request.1))
                            .expect("failed to send OpenOrders to ExchangeClient");
                    });
                }
                ExecutionRequest::CancelOrders(cancel_requests) => {
                    cancel_requests.into_iter().for_each(|cancel_request| {
                        let client = self.client(&cancel_request.0);
                        (*client)
                            .send(ExchangeRequest::CancelOrders(cancel_request.1))
                            .expect("failed to send CancelOrders to ExchangeClient");
                    });
                }
                ExecutionRequest::CancelOrdersAll(exchanges) => {
                    exchanges.into_iter().for_each(|exchange| {
                        let client = self.client(&exchange);
                        (*client)
                            .send(ExchangeRequest::CancelOrdersAll)
                            .expect("failed to send CancelOrdersAll to ExchangeClient");
                    });
                }
            }
        }
    }

    /// Retrieve the [`ExchangeClient`] associated with the [`Exchange`].
    pub fn client(&mut self, exchange: &Exchange) -> &mpsc::UnboundedSender<ExchangeRequest> {
        self.clients
            .get(exchange)
            .expect("cannot retrieve ExchangeClient for unexpected Exchange")
    }
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
