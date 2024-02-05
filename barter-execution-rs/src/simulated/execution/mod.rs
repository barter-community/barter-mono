use crate::{
    fill::Fees,
    model::{
        order::{Cancelled, Open, Order},
        order_event::OrderEvent,
    },
    simulated::SimulatedEvent,
    ExecutionClient, ExecutionError, ExecutionId, RequestCancel, RequestOpen, SymbolBalance,
};
use async_trait::async_trait;
use barter_integration::model::Exchange;
use tokio::sync::{mpsc, oneshot};

/// Simulated [`ExecutionClient`] implementation that integrates with the Barter
/// [`SimulatedExchange`](super::exchange::SimulatedExchange).
#[derive(Debug)]
pub struct SimulatedExecution {
    /// Simulated fee percentage to be used for each [`Fees`] field in decimal form (eg/ 0.01 for 1%)
    pub fees_pct: Fees,
    pub request_tx: mpsc::UnboundedSender<SimulatedEvent>,
}

/// Config for initializing a [`SimulatedExecution`] instance.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Simulated fee percentage to be used for each [`Fees`] field in decimal form (eg/ 0.01 for 1%)
    pub simulated_fees_pct: Fees,
    pub request_tx: mpsc::UnboundedSender<SimulatedEvent>,
}

#[async_trait]
impl ExecutionClient for SimulatedExecution {
    type Config = SimulationConfig;

    async fn init(config: Self::Config) -> Self {
        Self {
            request_tx: config.request_tx,
            fees_pct: config.simulated_fees_pct,
        }
    }

    fn exchange(&self) -> Exchange {
        Exchange::from(ExecutionId::Simulated)
    }

    // async fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
    //     // Assume (for now) that all orders are filled at the market price
    //     let fill_value_gross = SimulatedExecution::calculate_fill_value_gross(order);

    //     Ok(FillEvent {
    //         time: Utc::now(),
    //         exchange: order.exchange.clone(),
    //         instrument: order.instrument.clone(),
    //         market_meta: order.market_meta,
    //         decision: order.decision,
    //         quantity: order.quantity,
    //         fill_value_gross,
    //         fees: self.calculate_fees(&fill_value_gross),
    //     })
    // }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send FetchOrdersOpen request to the SimulatedExchange
        self.request_tx
            .send(SimulatedEvent::FetchOrdersOpen(response_tx))
            .expect("SimulatedExchange is offline - failed to send FetchOrdersOpen request");

        // Receive FetchOrdersOpen response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive FetchOrdersOpen response")
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send FetchBalances request to the SimulatedExchange
        self.request_tx
            .send(SimulatedEvent::FetchBalances(response_tx))
            .expect("SimulatedExchange is offline - failed to send FetchBalances request");

        // Receive FetchBalances response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive FetchBalances response")
    }

    async fn open_orders(
        &self,
        open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send OpenOrders request to the SimulatedExchange
        self.request_tx
            .send(SimulatedEvent::OpenOrders((open_requests, response_tx)))
            .expect("SimulatedExchange is offline - failed to send OpenOrders request");

        // Receive OpenOrders response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive OpenOrders response")
    }

    async fn cancel_orders(
        &self,
        cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send CancelOrders request to the SimulatedExchange
        self.request_tx
            .send(SimulatedEvent::CancelOrders((cancel_requests, response_tx)))
            .expect("SimulatedExchange is offline - failed to send CancelOrders request");

        // Receive CancelOrders response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive CancelOrders response")
    }

    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError> {
        // Oneshot channel to communicate with the SimulatedExchange
        let (response_tx, response_rx) = oneshot::channel();

        // Send CancelOrdersAll request to the SimulatedExchange
        self.request_tx
            .send(SimulatedEvent::CancelOrdersAll(response_tx))
            .expect("SimulatedExchange is offline - failed to send CancelOrdersAll request");

        // Receive CancelOrdersAll response from the SimulatedExchange
        response_rx
            .await
            .expect("SimulatedExchange is offline - failed to receive CancelOrdersAll response")
    }
}

impl SimulatedExecution {
    /// Calculates the simulated gross fill value (excluding TotalFees) based on the input [`OrderEvent`].
    pub fn calculate_fill_value_gross(order: &OrderEvent) -> f64 {
        order.quantity.abs() * order.market_meta.close
    }

    /// Calculates the simulated [`Fees`] a [`FillEvent`] will incur, based on the input [`OrderEvent`].
    pub fn calculate_fees(&self, fill_value_gross: &f64) -> Fees {
        Fees {
            exchange: self.fees_pct.exchange * fill_value_gross,
            slippage: self.fees_pct.slippage * fill_value_gross,
            network: self.fees_pct.network * fill_value_gross,
        }
    }
}

#[cfg(test)]
mod tests {
    use barter_integration::model::{
        instrument::{kind::InstrumentKind, Instrument},
        Exchange,
    };
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        fill::{Decision, MarketMeta},
        model::order_event::OrderType,
    };

    use super::*;

    /// Build an [`OrderEvent`] to buy 1.0 contract.
    pub fn order_event() -> OrderEvent {
        OrderEvent {
            id: Uuid::new_v4(),
            time: Utc::now(),
            exchange: Exchange::from("binance"),
            instrument: Instrument::from(("eth", "usdt", InstrumentKind::Spot)),
            market_meta: MarketMeta::default(),
            decision: Decision::default(),
            quantity: 1.0,
            order_type: OrderType::default(),
        }
    }

    // #[tokio::test]
    // async fn should_generate_ok_fill_event_with_valid_order_event_provided() {
    //     let simulated_execution = SimulatedExecution::init(
    //         SimulationConfig {
    //             simulated_fees_pct: Fees {
    //                 exchange: 0.1,
    //                 slippage: 0.05,
    //                 network: 0.0,
    //             },
    //             request_tx: mpsc::unbounded_channel().0,
    //         },
    //         mpsc::unbounded_channel().0,
    //     )
    //     .await;

    //     let mut input_order = order_event();
    //     input_order.quantity = 10.0;
    //     input_order.market_meta.close = 10.0;

    //     let actual_result = simulated_execution.generate_fill(&input_order).await;

    //     let expected_fill_value_gross = 100.0;
    //     let expected_fees = Fees {
    //         exchange: 10.0,
    //         slippage: 5.0,
    //         network: 0.0,
    //     };

    //     assert!(actual_result.is_ok());
    //     let actual_result = actual_result.unwrap();
    //     assert_eq!(actual_result.fill_value_gross, expected_fill_value_gross);
    //     assert_eq!(actual_result.fees, expected_fees);
    // }

    #[test]
    fn should_calculate_fill_value_gross_correctly() {
        let mut input_order = order_event();
        input_order.quantity = 100.0;
        input_order.market_meta.close = 10.0;

        let actual = SimulatedExecution::calculate_fill_value_gross(&input_order);

        let expected = 100.0 * 10.0;

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_calculate_fill_value_gross_correctly_with_negative_order_quantity_provided() {
        let mut input_order = order_event();
        input_order.quantity = -(100.0);
        input_order.market_meta.close = 10.0;

        let actual = SimulatedExecution::calculate_fill_value_gross(&input_order);

        let expected = (100.0 * 10.0) as f64;

        assert_eq!(actual, expected)
    }

    #[tokio::test]
    async fn should_calculate_simulated_fees_correctly() {
        let simulated_execution = SimulatedExecution::init(SimulationConfig {
            simulated_fees_pct: Fees {
                exchange: 0.5,
                slippage: 0.1,
                network: 0.001,
            },
            request_tx: mpsc::unbounded_channel().0,
        })
        .await;

        let input_fill_value_gross = 100.0;

        let actual_result = simulated_execution.calculate_fees(&input_fill_value_gross);

        let expected = Fees {
            exchange: 50.0,
            slippage: 10.0,
            network: 0.1,
        };

        assert_eq!(actual_result, expected)
    }
}
