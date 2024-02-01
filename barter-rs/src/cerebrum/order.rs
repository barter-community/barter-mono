use super::{consume::Consumer, exchange::ExecutionRequest, Cerebrum, Engine};

/// OrderGenerator can transition to:
///  a) Consumer
#[derive(Debug, Clone, Copy)]
pub struct OrderGenerator<State> {
    pub state: State,
}

#[derive(Debug, Clone, Copy)]
pub struct Algorithmic;

#[derive(Debug, Clone, Copy)]
pub struct Manual;

impl<Strategy> Cerebrum<OrderGenerator<Algorithmic>, Strategy>
where
    Strategy: super::strategy::OrderGenerator,
{
    pub fn generate_order_requests(self) -> Engine<Strategy> {
        // Send CancelOrders Command to ExchangeClient
        if let Some(cancel_requests) = self.strategy.generate_cancels() {
            self.request_tx
                .send(ExecutionRequest::CancelOrders(cancel_requests))
                .unwrap()
        }

        // Send OpenOrders Command to ExchangeClient
        if let Some(open_requests) = self.strategy.generate_orders() {
            self.request_tx
                .send(ExecutionRequest::OpenOrders(open_requests))
                .unwrap();
        }

        Engine::Consumer(Cerebrum::from(self))
    }
}

impl<Strategy> Cerebrum<OrderGenerator<Manual>, Strategy> {
    pub fn generate_order_requests_manual(self, meta: ()) -> Engine<Strategy> {
        // Todo:
        // 1. Action manual open / cancel order
        Engine::Consumer(Cerebrum::from(self))
    }
}

/// a) OrderGenerator -> Consumer
impl<State, Strategy> From<Cerebrum<OrderGenerator<State>, Strategy>>
    for Cerebrum<Consumer, Strategy>
{
    fn from(cerebrum: Cerebrum<OrderGenerator<State>, Strategy>) -> Self {
        Self {
            state: Consumer,
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}
