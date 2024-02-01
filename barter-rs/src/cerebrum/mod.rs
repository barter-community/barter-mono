use self::{
    account::{AccountUpdater, Accounts},
    command::Commander,
    consume::Consumer,
    event::{AccountEvent, EventFeed},
    exchange::ExecutionRequest,
    initialise::Initialiser,
    market::MarketUpdater,
    order::{Algorithmic, Manual, OrderGenerator},
    strategy::IndicatorUpdater,
    terminate::Terminated,
};
use crate::engine::error::EngineError;
use barter_data::event::{DataKind, MarketEvent};
use tokio::sync::mpsc;

pub mod account;
pub mod command;
pub mod consume;
pub mod event;
pub mod exchange;
pub mod initialise;
pub mod market;
pub mod order;
pub mod strategy;
pub mod terminate;

// Todo:
//  - Could have a thread for each exchange that processes MarketEvents -> Indicators/Statistics
//   '--> Those Indicators/Statistics are then the input into the system (if there are performance issues)
//  - Derive as eagerly as possible
//  - Add metric_tx stub?
//  - Determine what fields go in what state later
//  - Will need some startup States to go from New -> Initialised
//   '--> logic to hit the ExchangeClient to get balances, orders, positions.
//     '--> Start off with one ExchangeClient before adding many exchanges...
//     '--> exchange_tx / execution_tx / account_tx (or similar) to send Requests to exchange
//  - Make input & output feed / tx / rx names more distinct eg/ InputEventFeed, or InputFeed...
//     ... output_tx / audit_tx / state_tx etc
//  - Consumer state can likely transition to Initialiser while we wait for responses from exchange?
//  - Feed needs some work to be more like MarketFeed w/ Feed struct? etc.
//  - Engine will also have control of spawning the execution clients, presumably...?
//   '--> Perhaps Engine will need to be a struct and enum Engine -> CerebrumState/TradingState/Trader
//   '--> Perhaps the builder could do the Init of Cerebrum in a blocking way
//  - Is it valid to have a SymbolBalance, or do we need the idea of SymbolInstrumentKindBalance?
//  - Update Balances can be more efficient since we know what Markets we trade at the start
//   '--> Change update_balance() .expect() for error! like open order logic
//   '--> Can ignore anything which contains_key() returns None etc
//  - Work out how to do fees for trade, and add Liquidity field?
//  - Impl display for MarketEvent, AccountEvent, Command
//  - Could make Account generic to give it functionality to generate appropriate Cid?
//  - self.accounts.update_orders_from_open(&order); is taking ref & cloning - only makes sense if
//    we are using audit_tx... double check this later
//  - Should I have the concept & tracking of orders_in_flight_cancel? Along with associated State InFlightCancel?
//  - Account should probably have an Exchange -> perhaps Accounts<'a>(HashMap<&'a Exchange, Account>)
//  - Add states to transition to when we go unhealthy via ConnectionStatus eg/ CancelOnly, Offline etc.
//   '--> Can we add the MarketFeed health into this also? Want the MarketFeed to send ConnectionStatus / health
//        to EventFeed also, which can alter the EngineState
//  - Perhaps the EventFeed should just be a std::mpsc::unbounded? or crossbeam etc.
//  - Engine probably contains handles to the ExchangePortal, etc. Builder could do all of init...
//  - Would the idea of an ExchangeId::Simulated(u8) be satisfactory rather than Exchange?
//  - Make as much stuff reference as possible, eg/ Accounts could use reference Accounts<'a>(HashMap<&'a Symbol...)
//  - Make ExchangePortal generic so an Engine can be select with a higher performance portal for
//    a single Exchange only :) Same goes for all other multi-exchange functionality...
//  - InFlight cancels, but would need to generate CancelClientOrderId
//   '--> OrderManager { fn add_new_in_flight_cancels(&mut self, exchange: &Exchange, in_flight: Vec<Order<InFlightCancel>>);
//   '--> Account { cancels_in_flight: HashMap<ClientOrderId, Order<InFlightCancel>> }
//  - Should I be generating more errors...? eg/ OrderManager trait

#[derive(Debug)]
pub struct Components<Strategy> {
    feed: EventFeed,
    accounts: Accounts,
    exchange_tx: mpsc::UnboundedSender<ExecutionRequest>,
    strategy: Strategy,
    audit_tx: (),
}

#[derive(Debug)]
pub enum Engine<Strategy> {
    Initialiser(Cerebrum<Initialiser, Strategy>),
    Consumer(Cerebrum<Consumer, Strategy>),
    MarketUpdater((Cerebrum<MarketUpdater, Strategy>, MarketEvent<DataKind>)),
    OrderGeneratorAlgorithmic(Cerebrum<OrderGenerator<Algorithmic>, Strategy>),
    OrderGeneratorManual((Cerebrum<OrderGenerator<Manual>, Strategy>, ())),
    AccountUpdater((Cerebrum<AccountUpdater, Strategy>, AccountEvent)),
    Commander(Cerebrum<Commander, Strategy>),
    Terminated(Cerebrum<Terminated, Strategy>),
}

#[derive(Debug)]
pub struct Cerebrum<State, Strategy> {
    pub state: State,
    pub feed: EventFeed,
    pub accounts: Accounts,
    pub request_tx: mpsc::UnboundedSender<ExecutionRequest>,
    pub strategy: Strategy,
    pub audit_tx: (),
}

impl<Strategy> Engine<Strategy>
where
    Strategy: IndicatorUpdater + strategy::OrderGenerator,
{
    pub fn new(components: Components<Strategy>) -> Self {
        Self::Initialiser(Cerebrum {
            state: Initialiser,
            feed: components.feed,
            accounts: components.accounts,
            request_tx: components.exchange_tx,
            strategy: components.strategy,
            audit_tx: components.audit_tx,
        })
    }

    pub fn builder() -> EngineBuilder<Strategy> {
        EngineBuilder::new()
    }

    pub fn run(mut self) {
        'trading: loop {
            // Transition to the next trading state
            self = self.next();

            if let Self::Terminated(_) = self {
                // Todo: Print trading session results & persist
                break 'trading;
            }
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Initialiser(cerebrum) => cerebrum.init(),
            Self::Consumer(cerebrum) => cerebrum.next_event(),
            Self::MarketUpdater((cerebrum, market)) => cerebrum.update(market),
            Self::OrderGeneratorAlgorithmic(cerebrum) => cerebrum.generate_order_requests(),
            Self::OrderGeneratorManual((cerebrum, meta)) => {
                cerebrum.generate_order_requests_manual(meta)
            }
            Self::AccountUpdater((cerebrum, account)) => cerebrum.update(account),
            Self::Commander(cerebrum) => cerebrum.execute_manual_command(),
            Self::Terminated(cerebrum) => Self::Terminated(cerebrum),
        }
    }
}

/// Builder to construct [`Engine`] instances.
#[derive(Default, Debug)]
pub struct EngineBuilder<Strategy> {
    pub feed: Option<EventFeed>,
    pub accounts: Option<Accounts>,
    pub exchange_tx: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub strategy: Option<Strategy>,
    pub audit_tx: Option<()>,
}

impl<Strategy> EngineBuilder<Strategy> {
    fn new() -> Self {
        Self {
            feed: None,
            accounts: None,
            exchange_tx: None,
            strategy: None,
            audit_tx: None,
        }
    }

    pub fn feed(self, value: EventFeed) -> Self {
        Self {
            feed: Some(value),
            ..self
        }
    }

    pub fn accounts(self, value: Accounts) -> Self {
        Self {
            accounts: Some(value),
            ..self
        }
    }

    pub fn exchange_tx(self, value: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            exchange_tx: Some(value),
            ..self
        }
    }

    pub fn strategy(self, value: Strategy) -> Self {
        Self {
            strategy: Some(value),
            ..self
        }
    }

    pub fn audit_tx(self, value: ()) -> Self {
        Self {
            audit_tx: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<Engine<Strategy>, EngineError> {
        Ok(Engine::Initialiser(Cerebrum {
            state: Initialiser,
            feed: self
                .feed
                .ok_or(EngineError::BuilderIncomplete("engine_id"))?,
            accounts: self
                .accounts
                .ok_or(EngineError::BuilderIncomplete("account"))?,
            request_tx: self
                .exchange_tx
                .ok_or(EngineError::BuilderIncomplete("exchange_tx"))?,
            strategy: self
                .strategy
                .ok_or(EngineError::BuilderIncomplete("strategy"))?,
            audit_tx: self
                .audit_tx
                .ok_or(EngineError::BuilderIncomplete("audit_tx"))?,
        }))
    }
}
