use barter_data::event::{DataKind, MarketEvent};
use barter_execution::model::order::{Order, RequestCancel, RequestOpen};
use barter_integration::model::Exchange;

pub trait IndicatorUpdater {
    fn update_indicators(&mut self, market: &MarketEvent<DataKind>);
}

// Todo:
//  - Name clashes with OrderGenerator<State>
//  - Do I want two seperate states, one for generate_cancel(), one for generate_orders()?

pub trait OrderGenerator {
    fn generate_cancels(&mut self) -> Option<Vec<(Exchange, Vec<Order<RequestCancel>>)>>;
    fn generate_orders(&mut self) -> Option<Vec<(Exchange, Vec<Order<RequestOpen>>)>>;
}

// Todo: What does the Strategy do?
// - Updates Indicators
// - Analyses Indicators, in conjunction with Statistics, Positions, and Orders
// - Based on analysis, generates optional Order<Request>
// - Allocates Order<Request>
// - Decides Order<Request> OrderKind

// Todo  Strategy needs a view into accounts, but it should not do the account keeping
// perhaps Impl Strategy for Cerebrum...?
// perhaps struct Strategy<IndicatorUpdater, OrderGenerator>
