use super::{
    account::AccountUpdater,
    command::Commander,
    event::{Command, Event},
    market::MarketUpdater,
    Cerebrum, Engine,
};

/// Consumer can transition to one of:
///  a) MarketUpdater
///  b) AccountUpdater
///  c) Commander

#[derive(Debug, Clone, Copy)]
pub struct Consumer;

impl<Strategy> Cerebrum<Consumer, Strategy> {
    pub fn next_event(mut self) -> Engine<Strategy> {
        // Consume next Event
        match self.feed.next() {
            Event::Market(market) => Engine::MarketUpdater((Cerebrum::from(self), market)),
            Event::Account(account) => Engine::AccountUpdater((Cerebrum::from(self), account)),
            Event::Command(command) => Engine::Commander(Cerebrum::from((self, command))),
        }
    }
}

/// a) Consumer -> MarketUpdater
impl<Strategy> From<Cerebrum<Consumer, Strategy>> for Cerebrum<MarketUpdater, Strategy> {
    fn from(cerebrum: Cerebrum<Consumer, Strategy>) -> Self {
        Self {
            state: MarketUpdater,
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}

/// b) Consumer -> AccountUpdater
impl<Strategy> From<Cerebrum<Consumer, Strategy>> for Cerebrum<AccountUpdater, Strategy> {
    fn from(cerebrum: Cerebrum<Consumer, Strategy>) -> Self {
        Self {
            state: AccountUpdater,
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}

/// c) Consumer -> Commander
impl<Strategy> From<(Cerebrum<Consumer, Strategy>, Command)> for Cerebrum<Commander, Strategy> {
    fn from((cerebrum, command): (Cerebrum<Consumer, Strategy>, Command)) -> Self {
        Self {
            state: Commander { command },
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}
