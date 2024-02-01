use super::{consume::Consumer, terminate::Terminated, Cerebrum, Engine};

/// Initialiser can transition to one of:
///  a) Consumer
///  b) Terminated

#[derive(Debug, Clone, Copy)]
pub struct Initialiser;

impl<Strategy> Cerebrum<Initialiser, Strategy> {
    pub fn init(mut self) -> Engine<Strategy> {
        // Todo:
        //  - Or we do we this in the Builder? Perhaps...
        //  - Should this be 'AccountInitialisation'?
        //  - There should be an initialisation timeout
        //  - Hit ExchangeClient to get balances, orders, positions (may fail)
        //  - Add failure transition to Engine::Terminated if it's unrecoverable

        // Process:
        // 1. Ask ExchangePortal for ConnectionStatus of every ExchangeClient & react
        //   '--> ExchangeClient may still be starting up, perhaps have a timeout.
        // 2. Once Online, ask ExchangeClient for Balances & Orders
        // 3. Wait for responses with timeouts
        // 4. Use responses to populate Accounts
        // loop {
        //     match self.feed.next() {
        //         Event::Account(account) => {
        //             break Engine::Consumer(Cerebrum::from(self))
        //         }
        //         Event::Command(Command::Terminate) => {
        //             break Engine::Terminated(Cerebrum::from(self))
        //         }
        //         _ => continue
        //     }
        // }

        Engine::Consumer(Cerebrum::from(self))
    }
}

/// a) Initialiser -> Consumer
impl<Strategy> From<Cerebrum<Initialiser, Strategy>> for Cerebrum<Consumer, Strategy> {
    fn from(cerebrum: Cerebrum<Initialiser, Strategy>) -> Self {
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

/// b) Initialiser -> Terminated
impl<Strategy> From<Cerebrum<Initialiser, Strategy>> for Cerebrum<Terminated, Strategy> {
    fn from(cerebrum: Cerebrum<Initialiser, Strategy>) -> Self {
        Self {
            state: Terminated,
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}
