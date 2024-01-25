use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub outputs: Vec<Output>,
    #[serde(rename = "encodedOrder")]
    pub encoded_order: String,
    pub signature: String,
    pub input: Input,
    #[serde(rename = "settledAmounts")]
    pub settled_amounts: Option<Vec<SettledAmount>>,
    #[serde(rename = "orderStatus")]
    pub order_status: String,
    #[serde(rename = "txHash")]
    pub tx_hash: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    #[serde(rename = "orderHash")]
    pub order_hash: String,
    #[serde(rename = "type")]
    pub order_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    recipient: String,
    #[serde(rename = "startAmount")]
    start_amount: String,
    #[serde(rename = "endAmount")]
    end_amount: String,
    token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    #[serde(rename = "endAmount")]
    end_amount: String,
    token: String,
    #[serde(rename = "startAmount")]
    start_amount: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettledAmount {
    #[serde(rename = "tokenOut")]
    token_out: String,
    #[serde(rename = "amountIn")]
    amount_in: String,
    #[serde(rename = "amountOut")]
    amount_out: String,
    #[serde(rename = "tokenIn")]
    token_in: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
  pub orders: Vec<Order>,
}