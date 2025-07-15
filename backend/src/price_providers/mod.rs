mod binance_price_provider;

use binance_price_provider::binance_api::{BinanceAPI, AggTradesResponse};
use chrono::{DateTime, Duration, Utc};

pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
}
pub type PriceSeries = Vec<PricePoint>;

pub struct BinancePriceProvider {
    binance_api: Box<dyn BinanceAPI>,
}

impl BinancePriceProvider {
    const TIME_WINDOW: Duration = Duration::minutes(1);

    pub fn new(binance_api: Box <dyn BinanceAPI>) -> BinancePriceProvider {
        BinancePriceProvider{ binance_api }
    }

    fn fetch_avg_price_for_window(&self, symbol: &str, window_start: &DateTime<Utc>, window_end: &DateTime<Utc>) -> anyhow::Result<Option<f64>> {
        let api_response = self.binance_api.agg_trades(
            symbol,
            None,
            Some( window_start.timestamp_millis() ),
            Some( window_end.timestamp_millis() ),
            None)?;
        let response_json: AggTradesResponse = serde_json::from_str(&api_response)?;

        let response_prices: Vec<f64> = response_json
            .iter()
            .map(|trade| trade.p.parse::<f64>())
            .collect::<Result<Vec<f64>, _>>()?;

        let sum = response_prices.iter().sum::<f64>();
        let count = response_prices.len() as f64;
        
        if !response_prices.is_empty() { Ok(Some(sum / count)) } else { Ok(None) }
    }

    pub fn prices(&self, symbol: &str, start_time: &DateTime<Utc>, end_time: &DateTime<Utc>) -> anyhow::Result<PriceSeries> {
        let mut prices = Vec::new();
        let window_starts = std::iter::successors(Some(*start_time), |prev| {
            let next = *prev + Self::TIME_WINDOW;
            if next < *end_time { Some(next) } else { None }
        });
        for window_start in window_starts {
            let window_end = std::cmp::min(
                window_start + Self::TIME_WINDOW - Duration::milliseconds(1),
                *end_time);
            match self.fetch_avg_price_for_window(&window_start, &window_end)? {
                Some(avg_price) => {
                prices.push(PricePoint { timestamp: window_start, price: avg_price });
                },
                None => {},
            }
        }
        Ok(prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use mockall::mock;
    use mockall::predicate::*;
    extern crate assert_float_eq;
    use assert_float_eq::assert_float_absolute_eq;
    use chrono::prelude::*;
    use std::sync::LazyLock;

    mock! {
        BinanceAPI {}
        impl BinanceAPI for BinanceAPI {
            fn agg_trades(&self, 
                          symbol: &str,
                          from_id: Option<i64>,
                          start_time: Option<i64>,
                          end_time: Option<i64>,
                          limit: Option<i64>) -> anyhow::Result<String>;
        }
    }

    const SYMBOL: &'static str = "BTCUSDC";

    const SINGLE_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"p": "0.01633102","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;
    const MULTIPLE_PRICES_RESPONSE: &'static str = concat!(
        r#"[{"a": 26129,"p": "1.0","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true },"#,
        r#"{"a": 26129,"p": "2.5","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true },"#,
        r#"{"a": 26129,"p": "3.5","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#
    );
    const MULTIPLE_PRICES_RESPONSE_2: &'static str = concat!(
        r#"[{"a": 26129,"p": "1.0","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true },"#,
        r#"{"a": 26129,"p": "2.0","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#,
    );
    const MISSING_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;
    const INVALID_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"p": "notafloat","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;

    // Default time values spaning just one time window
    const START_TIME: LazyLock<DateTime<Utc>> = LazyLock::new( || 
        Utc.with_ymd_and_hms(2025,1,27,14,0,0).unwrap() );
    const END_TIME: LazyLock<DateTime<Utc>> = LazyLock::new( || 
        *START_TIME + BinancePriceProvider::TIME_WINDOW - Duration::seconds(1) );    

    #[test]
    fn test_can_create_a_binance_price_provider() {
        let mock_api = MockBinanceAPI::new();
        let _binance_provider = BinancePriceProvider::new(Box::new(mock_api));
    }

    #[test]
    fn test_binance_provider_returns_empty_when_no_prices() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .times(1)
            .with(
                eq(SYMBOL), 
                always(), 
                always(), 
                always(), 
                always())
            .returning(|_,_,_,_,_| Ok("[]".to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let prices = binance_provider.prices(SYMBOL, &START_TIME, &END_TIME);

        assert!( prices.is_ok() );
        assert!( prices.unwrap().is_empty() );
    }

    #[test]
    fn test_binance_provider_returns_price_if_just_one_price() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .times(1)
            .with(
                eq(SYMBOL), 
                always(), 
                always(), 
                always(), 
                always())
            .returning(|_,_,_,_,_| Ok(SINGLE_PRICE_RESPONSE.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let prices = binance_provider.prices(SYMBOL, &START_TIME, &END_TIME).unwrap();
        
        assert_eq!( prices.len(), 1 );
        assert_float_absolute_eq!( prices[0].price, 0.01633102 );
    }

    #[test]
    fn test_binance_provider_returns_error_on_api_error() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .returning(|_,_,_,_,_| Err(anyhow::Error::msg("some error")));
        
        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        assert!( binance_provider.prices(SYMBOL, &START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_error_on_missing_price_data() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .returning(|_,_,_,_,_| Ok(MISSING_PRICE_RESPONSE.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        assert!( binance_provider.prices(SYMBOL, &START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_error_on_non_numeric_price_data() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .returning(|_,_,_,_,_| Ok(INVALID_PRICE_RESPONSE.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        assert!( binance_provider.prices(SYMBOL, &START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_average_price_from_single_time_window() {
        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .times(1)
            .with(
                eq(SYMBOL), 
                always(), 
                eq(Some(START_TIME.timestamp_millis())), 
                eq(Some(END_TIME.timestamp_millis())), 
                always())
            .returning(|_,_,_,_,_| Ok(MULTIPLE_PRICES_RESPONSE.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let prices = binance_provider.prices(SYMBOL, &START_TIME, &END_TIME).unwrap();

        assert_eq!( prices.len(), 1 );
        assert_float_absolute_eq!( prices[0].price, 2.333333333 );
        assert_eq!( prices[0].timestamp, *START_TIME );
    }

    #[test]
    fn test_binance_provider_returns_average_prices_from_multiple_time_windows() {
        // ensure to capture just 2 windows
        let first_window_end = *START_TIME + BinancePriceProvider::TIME_WINDOW;
        let end_time = first_window_end + Duration::seconds(1);

        let mut mock_api = MockBinanceAPI::new();
        // call #1
        mock_api.expect_agg_trades()
            .times(1)
            .with(
                eq(SYMBOL), 
                always(), 
                eq(Some(START_TIME.timestamp_millis())), 
                eq(Some(first_window_end.timestamp_millis()-1)), 
                always() )
            .returning(|_,_,_,_,_| Ok(MULTIPLE_PRICES_RESPONSE.to_string()));
        // call #2
        mock_api.expect_agg_trades()
        .times(1)
        .with(
            eq(SYMBOL), 
            always(), 
            eq(Some(first_window_end.timestamp_millis())), 
            eq(Some(end_time.timestamp_millis())), 
            always() )
        .returning(|_,_,_,_,_| Ok(MULTIPLE_PRICES_RESPONSE_2.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let prices = binance_provider.prices(SYMBOL, &START_TIME, &end_time).unwrap();
        
        assert_eq!( prices.len(), 2 );
        assert_float_absolute_eq!( prices[0].price, 2.333333333 );
        assert_eq!( prices[0].timestamp, *START_TIME );
        assert_float_absolute_eq!( prices[1].price, 1.5 );
        assert_eq!( prices[1].timestamp, first_window_end );
    }

    #[test]
    fn test_binance_provider_skips_time_windows_with_no_prices() {
       // capture 3 windows
       let end_time = *START_TIME + BinancePriceProvider::TIME_WINDOW * 3;

       let mut mock_api = MockBinanceAPI::new();
       // call #1
       mock_api.expect_agg_trades()
           .times(1)
           .returning(|_,_,_,_,_| Ok(MULTIPLE_PRICES_RESPONSE.to_string()));
        // call #2
        mock_api.expect_agg_trades()
        .times(1)
        .returning(|_,_,_,_,_| Ok("[]".to_string()));
        // call #3
        mock_api.expect_agg_trades()
           .times(1)
           .returning(|_,_,_,_,_| Ok(MULTIPLE_PRICES_RESPONSE_2.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let prices = binance_provider.prices(SYMBOL, &START_TIME, &end_time).unwrap();
        
        assert_eq!( prices.len(), 2 );
        assert_float_absolute_eq!( prices[0].price, 2.333333333 );
        assert_eq!( prices[0].timestamp, *START_TIME );
        assert_float_absolute_eq!( prices[1].price, 1.5 );
        assert_eq!( prices[1].timestamp, *START_TIME + BinancePriceProvider::TIME_WINDOW * 2 );
    }

    #[test]
    fn test_binance_provider_returns_prices_for_given_symbol() {
        const NEW_SYMBOL: &'static str = "ETHUSDT";

        let mut mock_api = MockBinanceAPI::new();
        mock_api.expect_agg_trades()
            .times(1)
            .with(
                eq(NEW_SYMBOL),
                always(),
                always(),
                always(),
                always(),
            )
            .returning(|_,_,_,_,_| Ok(SINGLE_PRICE_RESPONSE.to_string()));

        let binance_provider = BinancePriceProvider::new(Box::new(mock_api));
        let _ = binance_provider.prices(NEW_SYMBOL, &START_TIME, &END_TIME);
    }

}
