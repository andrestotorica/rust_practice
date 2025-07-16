use serde::Deserialize;

pub trait BinanceAPI { 
    /// GET /api/v3/aggTrades
    /// 
    /// Parameters
    /// symbol      STRING  YES    
    /// fromId      LONG    NO  ID to get aggregate trades from INCLUSIVE.
    /// startTime   LONG    NO  Timestamp in ms to get aggregate trades from INCLUSIVE.
    /// endTime     LONG    NO  Timestamp in ms to get aggregate trades until INCLUSIVE.
    /// limit       INT     NO  Default 500; max 1000.
    /// 
    /// Expected Response:
    /// [
    ///   {
    ///     "a": 26129,         // Aggregate tradeId
    ///     "p": "0.01633102",  // Price
    ///     "q": "4.70443515",  // Quantity
    ///     "f": 27781,         // First tradeId
    ///     "l": 27781,         // Last tradeId
    ///     "T": 1498793709153, // Timestamp
    ///     "m": true,          // Was the buyer the maker?
    ///     "M": true           // Was the trade the best price match?
    ///   }
    /// ]
    fn agg_trades(&self, 
        symbol: &str,
        from_id: Option<i64>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<String>;
}

#[derive(Deserialize)]
#[allow(non_snake_case,dead_code)]
pub struct AggTradesResponseItem {
    pub a: i64,                 
    pub p: String,
    pub q: String,
    pub f: i64,
    pub l: i64,
    pub T: i64,
    pub m: bool,
    pub M: bool,
}
pub type AggTradesResponse = Vec<AggTradesResponseItem>;


pub struct BinanceHttpClient {
    client: reqwest::blocking::Client,
    agg_trades_endpoint: String,
}

impl BinanceHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            agg_trades_endpoint: "https://api.binance.com/api/v3/aggTrades".to_string(),
        }
    }
}

impl BinanceAPI for BinanceHttpClient {

    fn agg_trades(&self, 
        symbol: &str,
        from_id: Option<i64>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<String> {

        let mut req = self.client.get(&self.agg_trades_endpoint)
            .query(&[("symbol", symbol)]);

        for (key, value) in [
            ("fromId", &from_id),
            ("startTime", &start_time),
            ("endTime", &end_time),
            ("limit", &limit),
        ] {
            if let Some(v) = value {
                req = req.query(&[(key, &v.to_string())]);
            }
        }

        let resp = req.send()?.error_for_status()?;

        let text = resp.text()?;
        Ok(text)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    impl BinanceHttpClient {
        pub fn new_with_test_endpoint() -> Self {
            Self {
                client: reqwest::blocking::Client::new(),
                agg_trades_endpoint: format!("{}/api/v3/aggTrades", &mockito::server_url()),
            }
        }
    }

    // TODO: since creating a test api with mockito is lightweight this tests don't make more sense
    // Need to use the mocked API on the BinancePriceProvider tests for more comprehensive testing
    // and get rid of these

    fn server_mock(return_status: usize, response: &str) -> mockito::Mock {
        mock("GET", "/api/v3/aggTrades")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbol".into(), "ETHUSDT".into()),
                Matcher::UrlEncoded("startTime".into(), "100".into()),
                Matcher::UrlEncoded("endTime".into(), "500".into()),
            ]))
            .with_status(return_status)
            .with_body(response)
            .create()
    }

    #[test]
    fn test_agg_trades_success() {
        let _m = server_mock(200, "a response");

        let client = BinanceHttpClient::new_with_test_endpoint();
        let result = client.agg_trades(
            "ETHUSDT", 
            None,
            Some(100),
            Some(500),
            None,
        );
        assert_eq!(result.unwrap(), "a response");
    }

    #[test]
    fn test_agg_trades_error() {
        let _m = server_mock(500, "Internal Server Error");

        let client = BinanceHttpClient::new_with_test_endpoint();
        let result = client.agg_trades(
            "ETHUSDT", 
            None,
            Some(100),
            Some(500),
            None,
        );
        assert!(result.is_err());
    }
}