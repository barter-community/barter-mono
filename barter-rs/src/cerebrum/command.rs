use crate::cerebrum::event::Command;
use crate::cerebrum::order::{Manual, OrderGenerator};
use crate::cerebrum::terminate::Terminated;
use crate::cerebrum::{Cerebrum, Engine};
use tracing::info;

/// Commander can transition to:
///  a) End
///  b) OrderGenerator<Manual>
#[derive(Debug, Clone, Copy)]
pub struct Commander {
    pub command: Command,
}

impl<Strategy> Cerebrum<Commander, Strategy> {
    pub fn execute_manual_command(self) -> Engine<Strategy> {
        // Action Command
        match self.state.command {
            Command::Terminate => {
                info!(kind = "Command", payload = "Terminate", "received Event");
                // Todo: Do pre-termination tasks
                Engine::Terminated(Cerebrum::from(self))
            }
            Command::FetchOpenPositions => {
                info!(
                    kind = "Command",
                    payload = "FetchOpenPositions",
                    "received Event"
                );
                // Todo: Send data to audit_tx
                Engine::Terminated(Cerebrum::from(self))
            }
            Command::ExitPosition => {
                info!(kind = "Command", payload = "ExitPosition", "received Event");
                // Todo: Add relevant metadata for the Position to exit
                Engine::OrderGeneratorManual((Cerebrum::from(self), ()))
            }
            Command::ExitAllPositions => {
                info!(
                    kind = "Command",
                    payload = "ExitAllPositions",
                    "received Event"
                );
                // Todo: Add relevant metadata for the Position to exit
                Engine::OrderGeneratorManual((Cerebrum::from(self), ()))
            }
        }
    }
}

/// a) Commander -> End
impl<Strategy> From<Cerebrum<Commander, Strategy>> for Cerebrum<Terminated, Strategy> {
    fn from(cerebrum: Cerebrum<Commander, Strategy>) -> Self {
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

/// b) Commander -> OrderGenerator<Manual>
impl<Strategy> From<Cerebrum<Commander, Strategy>> for Cerebrum<OrderGenerator<Manual>, Strategy> {
    fn from(cerebrum: Cerebrum<Commander, Strategy>) -> Self {
        Self {
            state: OrderGenerator { state: Manual },
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}
