use super::{order::Algorithmic, strategy::IndicatorUpdater, Cerebrum, Engine, OrderGenerator};
use barter_data::event::{DataKind, MarketEvent};
use tracing::info;

/// MarketUpdater can transition to:
///  a) OrderGenerator<Algorithmic>
#[derive(Debug, Clone, Copy)]
pub struct MarketUpdater;

impl<Strategy> Cerebrum<MarketUpdater, Strategy>
where
    Strategy: IndicatorUpdater,
{
    pub fn update(mut self, market: MarketEvent<DataKind>) -> Engine<Strategy> {
        // println!("MarketUpdater: {:?}", market);
        info!(kind = "Market", exchange = ?market.exchange, instrument = %market.instrument, payload = ?market, "received Event");

        // Update Positions
        self.accounts.update_positions(&market);

        // Update Indicators
        self.strategy.update_indicators(&market);

        Engine::OrderGeneratorAlgorithmic(Cerebrum::from(self))
    }
}

/// a) MarketUpdater -> OrderGenerator<Algorithmic>
impl<Strategy> From<Cerebrum<MarketUpdater, Strategy>>
    for Cerebrum<OrderGenerator<Algorithmic>, Strategy>
{
    fn from(cerebrum: Cerebrum<MarketUpdater, Strategy>) -> Self {
        Self {
            state: OrderGenerator { state: Algorithmic },
            feed: cerebrum.feed,
            accounts: cerebrum.accounts,
            request_tx: cerebrum.request_tx,
            strategy: cerebrum.strategy,
            audit_tx: cerebrum.audit_tx,
        }
    }
}
