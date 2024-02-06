use crate::dex::tokens::Token;

#[derive(Debug, Clone)]
pub struct Market {
    pub quotes: Vec<String>,
    pub quote: String,
    pub base: String,
    pub buy: bool,
}

impl Market {
    pub fn new(buy_token: &Token, sell_token: &Token) -> Market {
        let quotes = vec!["usdc", "usdt", "dai", "wbtc", "eth", "weth", "steth"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (quote, base, buy) =
            Market::get_quote_and_base(&quotes, &buy_token.symbol, &sell_token.symbol);

        Market {
            quotes,
            quote,
            base,
            buy,
        }
    }

    pub fn get_quote_and_base(
        quotes: &Vec<String>,
        buy_token: &str,
        sell_token: &str,
    ) -> (String, String, bool) {
        let buy_index = quotes.iter().position(|x| x == &buy_token.to_lowercase());
        let sell_index = quotes.iter().position(|x| x == &sell_token.to_lowercase());

        match (buy_index, sell_index) {
            (Some(bi), Some(si)) => {
                if bi < si {
                    (buy_token.to_string(), sell_token.to_string(), false)
                } else {
                    (sell_token.to_string(), buy_token.to_string(), true)
                }
            }
            (Some(_), None) => (buy_token.to_string(), sell_token.to_string(), false),
            (None, Some(_)) => (sell_token.to_string(), buy_token.to_string(), true),
            _ => {
                if buy_token < sell_token {
                    (buy_token.to_string(), sell_token.to_string(), false)
                } else {
                    (sell_token.to_string(), buy_token.to_string(), true)
                }
            }
        }
    }
}
