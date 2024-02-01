use super::{consume::Consumer, Cerebrum, Engine};
use barter_data::event::{DataKind, MarketEvent};
use barter_execution::model::{
    balance::{Balance, SymbolBalance},
    order::{Cancelled, InFlight, Open, Order},
    AccountEvent, AccountEventKind, ClientOrderId,
};
use barter_integration::model::{
    instrument::{symbol::Symbol, Instrument},
    Exchange,
};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// AccountUpdater can transition to:
///  a) Consumer
#[derive(Debug, Clone, Copy)]
pub struct AccountUpdater;

impl<Strategy> Cerebrum<AccountUpdater, Strategy> {
    pub fn update(mut self, account: AccountEvent) -> Engine<Strategy> {
        // Update Positions, Statistics, Indicators
        match account.kind {
            // AccountEventKind::ConnectionStatus(status) => {
            //     info!(kind = "Account", exchange = ?account.exchange, payload = ?status, "received Event");
            //     // Todo: React to ConnectionStatus
            // }
            AccountEventKind::Balance(balance) => {
                info!(kind = "Account", exchange = ?account.exchange, payload = ?balance, "received Event");
                self.accounts.update_balance(&account.exchange, &balance);
            }
            AccountEventKind::Balances(balances) => {
                info!(kind = "Account", exchange = ?account.exchange, payload = ?balances, "received Event");
                self.accounts.update_balances(&account.exchange, &balances);
            }

            AccountEventKind::OrdersOpen(orders) => {
                info!(kind = "Account", exchange = ?account.exchange, payload = "OrdersOpen", "received Event");
                orders
                    .iter()
                    .for_each(|order| self.accounts.update_orders_from_open(&order));
            }

            // TODO: do we need to treat OrdersNew differently to OrdersOpen?
            // inflight vs open?
            AccountEventKind::OrdersNew(orders) => {
                info!(kind = "Account", exchange = ?account.exchange, payload = "OrdersNew", "received Event");
                orders
                    .iter()
                    .for_each(|order| self.accounts.update_orders_from_open(&order));
                // .for_each(|order| self.accounts.update_order_from_new(&order));
            }

            AccountEventKind::OrdersCancelled(cancelled) => {
                info!(kind = "Account", exchange = ?account.exchange, payload = "OrderCancelled", "received Event");
                cancelled
                    .iter()
                    .for_each(|order| self.accounts.update_orders_from_cancel(&order));
            }

            AccountEventKind::Trade(trade) => {
                info!(kind = "Account", exchange = ?account.exchange, instrument = %trade.instrument, payload = ?trade, "received Event");
                // Todo: React to Trade... check for fully filled Orders, see update_from_fill(), etc.
            } // AccountEventKind::ExecutionError(error) => {
              //     error!(kind = "Account", exchange = ?account.exchange, payload = ?error, "received Event");
              // }
        };

        Engine::Consumer(Cerebrum::from(self))
    }
}

/// a) AccountUpdater -> Consumer
impl<Strategy> From<Cerebrum<AccountUpdater, Strategy>> for Cerebrum<Consumer, Strategy> {
    fn from(cerebrum: Cerebrum<AccountUpdater, Strategy>) -> Self {
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

#[derive(Debug)]
pub struct Accounts(pub HashMap<Exchange, Account>);

#[derive(Debug)]
pub struct Account {
    pub balances: HashMap<Symbol, Balance>,
    pub positions: HashMap<Instrument, Position>,
    pub orders_in_flight: HashMap<ClientOrderId, Order<InFlight>>,
    pub orders_open: HashMap<ClientOrderId, Order<Open>>,
}

impl Accounts {
    pub fn account(&mut self, exchange: &Exchange) -> &mut Account {
        self.0
            .get_mut(exchange)
            .expect("cannot retrieve Account for unexpected Exchange")
    }

    pub fn update_balance(&mut self, exchange: &Exchange, balance: &SymbolBalance) {
        self.account(exchange)
            .balances
            .get_mut(&balance.symbol)
            .and_then(|account_balance| {
                account_balance.total = balance.balance.total;
                account_balance.available = balance.balance.available;
                Some(account_balance)
            })
            .expect("cannot update Balance for unexpected Symbol");
    }

    pub fn update_balances(&mut self, exchange: &Exchange, balances: &Vec<SymbolBalance>) {
        balances
            .into_iter()
            .for_each(|balance| self.update_balance(exchange, balance))
    }

    pub fn update_positions(&mut self, market: &MarketEvent<DataKind>) {
        // Todo: Update relevant Positions
    }

    // Todo: refactor this if we don't use in_flight
    pub fn update_order_from_new(&mut self, order: &Order<InFlight>) {
        // Exchange Account associated with the Order
        let account = self.account(&order.exchange);

        match account.orders_in_flight.insert(order.cid, order.clone()) {
            Some(order_duplicate_cid) => {
                error!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    previous = ?order_duplicate_cid,
                    new = ?order,
                    action = "replacing previous Order<InFlight> with new one",
                    "received Order<InFlight> with duplicate cid to another in orders_in_flight"
                );
            }
            None => {
                debug!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "added to orders_in_flight HashMap",
                    "received Order<InFlight>"
                );
            }
        };
    }

    /// Update relevant [`Exchange`] [`Account`] after receiving an [`Order<Open>`].
    ///
    /// **Process:**
    /// a) Remove from orders_in_flight.
    /// b) Add to orders_open
    ///
    /// **Notes:**
    ///  - Expect that the [`Order<Open>`] is in the orders_in_flight HashMap.
    /// Todo: refactor this if we don't use in_flight
    pub fn update_orders_from_open(&mut self, order: &Order<Open>) {
        // Exchange Account associated with the Order
        let account = self.account(&order.exchange);

        match (
            account.orders_in_flight.remove(&order.cid),
            account.orders_open.insert(order.cid, order.clone()),
        ) {
            (Some(in_flight), Some(order_duplicate_cid)) => {
                error!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    in_flight = ?in_flight,
                    previous = ?order_duplicate_cid,
                    new = ?order,
                    action = "removing Order<InFlight> & replacing previous Order<Open> with new one",
                    "received Order<Open> for Order<Inflight>, with duplicate cid to another in orders_open"
                );
            }
            (None, None) => {
                warn!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "ignoring",
                    "received Order<Open> for Order not InFlight & not Open"
                );
            }
            (None, Some(order_duplicate_cid)) => {
                // Todo: This would be regular if we subscribe to WS AccountEvents & return HTTP orders
                warn!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    previous = ?order_duplicate_cid,
                    // new = ?order,
                    action = "replacing previous Order<Open> with new one",
                    "received Order<Open> with duplicate cid to another in orders_open"
                );
            }
            (Some(_), None) => {
                debug!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "removed from orders_in_flight HashMap",
                    "received Order<Open> for Order<InFlight>"
                );
            }
        };
    }

    /// Update relevant [`Exchange`] [`Account`] after receiving an [`Order<Cancelled>`].
    ///
    /// **Process:**
    /// a) Remove from orders_in_flight (if it's there).
    /// b) Remove from orders_open (if it's there).
    ///
    /// **Notes:**
    ///  - Possible that we receive an [`Order<Cancelled>`] before we receive an [`Order<Open>`], so
    ///    attempt to remove it from both HashMaps & log.
    ///  - It's expected that we will receive a separate [`Account`] [`Balance`] update relating to
    ///    the [`Order<Cancelled>`], therefore we do not alter the [`Balance`] HashMap.
    pub fn update_orders_from_cancel(&mut self, order: &Order<Cancelled>) {
        // Exchange Account associated with the Order
        let account = self.account(&order.exchange);

        // Expected Behaviour:
        //  - Order<Cancelled> is never in both orders_open and orders_in_flight HashMaps.
        //  - Order<Cancelled> most likely in orders_open, but could be in orders_in_flight.
        match (
            account.orders_open.remove(&order.cid),
            account.orders_in_flight.remove(&order.cid),
        ) {
            (Some(_), Some(_)) => {
                error!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "removed from both orders_in_flight & orders_open HashMaps",
                    "received Order<Cancelled> for Order InFlight and Open"
                );
            }
            (None, None) => {
                error!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "ignoring",
                    "received Order<Cancelled> for Order not InFlight & not Open"
                );
            }
            (None, Some(_)) => {
                warn!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "removed from orders_in_flight HashMap",
                    "received Order<Cancelled> for Order InFlight but not Open"
                );
            }
            (Some(_), None) => {
                debug!(
                    exchange = ?order.exchange,
                    cid = ?order.cid,
                    action = "removed from orders_open HashMap",
                    "received Order<Cancelled> for Order<Open>"
                );
            }
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position;
