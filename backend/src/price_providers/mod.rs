mod binance_price_provider;

use binance_price_provider::binance_api::{BinanceAPI, AggTradesResponse};
use chrono::{DateTime, Duration, Utc};

struct BinancePriceProvider {
    binance_api: Box<dyn BinanceAPI>,
}

impl BinancePriceProvider {
    const SYMBOL: &'static str = "BTCUSDC";
    const TIME_WINDOW: Duration = Duration::minutes(1);

    fn new(binance_api: Box <dyn BinanceAPI>) -> BinancePriceProvider {
        BinancePriceProvider{ binance_api }
    }

    fn prices(&self, start_time: &DateTime<Utc>, end_time: &DateTime<Utc>) -> anyhow::Result<Vec<f64>> {
        let api_response = self.binance_api.agg_trades(
            Self::SYMBOL,
            None,
            Some( start_time.timestamp_millis() ),
            Some( end_time.timestamp_millis() ),
            None)?;
        let response_json: AggTradesResponse = serde_json::from_str(&api_response)?;
        
        let mut count = 0;
        let sum = response_json
            .iter()
            .try_fold(0.0, |sum, trade| -> anyhow::Result<f64> {
                count += 1;
                let price = trade.p.parse::<f64>()?;
                Ok( sum + price )
            })?;

        if count == 0 {
            return Ok(vec![]);
        }
        Ok( vec![sum/count as f64] )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate assert_float_eq;
    use assert_float_eq::assert_float_absolute_eq;
    use chrono::prelude::*;
    use std::sync::LazyLock;

    // Default time values spaning just one time window
    const START_TIME: LazyLock<DateTime<Utc>> = LazyLock::new( || 
        Utc.with_ymd_and_hms(2025,1,27,14,0,0).unwrap() );
    const  END_TIME: LazyLock<DateTime<Utc>> = LazyLock::new( || 
        *START_TIME + BinancePriceProvider::TIME_WINDOW - Duration::seconds(1) );

    fn create_binance_provider_fixture(response: AggTradesCall) -> BinancePriceProvider {
        let b_api = FakeBinanceAPI{ response };
        BinancePriceProvider::new(Box::new(b_api))
    }
    
    #[test]
    fn test_can_create_a_binance_price_provider() {
        let api_call: AggTradesCall = Box::new( |_,_,_,_,_| Ok("[]".to_string()) );
        let _binance_provider = create_binance_provider_fixture(api_call);
    }

    #[test]
    fn test_binance_provider_returns_empty_when_no_prices() {
        let api_call: AggTradesCall = Box::new( |symbol,_,_,_,_| {
            assert_eq!( symbol, BinancePriceProvider::SYMBOL.to_string());
            Ok("[]".to_string())
        });
        let binance_provider = create_binance_provider_fixture(api_call);
        let prices = binance_provider.prices(&START_TIME, &END_TIME);
        assert!( prices.is_ok() );
        assert!( prices.unwrap().is_empty() );
    }

    #[test]
    fn test_binance_provider_returns_price_if_just_one_price() {
        let api_call: AggTradesCall = Box::new( |symbol,_,_,_,_| {
            assert_eq!( symbol, BinancePriceProvider::SYMBOL.to_string());
            Ok(FakeBinanceAPI::SINGLE_PRICE_RESPONSE.to_string())
        });
        let binance_provider = create_binance_provider_fixture(api_call);
        let prices = binance_provider.prices(&START_TIME, &END_TIME).unwrap();
        assert_eq!( prices.len(), 1 );
        assert_float_absolute_eq!( prices[0], 0.01633102 );
    }

    #[test]
    fn test_binance_provider_returns_error_on_api_error() {
        let api_call: AggTradesCall = Box::new( |_,_,_,_,_| Err( anyhow::Error::msg("some error") ));
        let binance_provider = create_binance_provider_fixture(api_call);
        assert!( binance_provider.prices(&START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_error_on_missing_price_data() {
        let api_call: AggTradesCall = Box::new( |_,_,_,_,_| Ok(FakeBinanceAPI::MISSING_PRICE_RESPONSE.to_string()));
        let binance_provider = create_binance_provider_fixture(api_call);
        assert!( binance_provider.prices(&START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_error_on_non_numeric_price_data() {
        let api_call: AggTradesCall = Box::new( |_,_,_,_,_| Ok(FakeBinanceAPI::INVALID_PRICE_RESPONSE.to_string()));
        let binance_provider = create_binance_provider_fixture(api_call);
        assert!( binance_provider.prices(&START_TIME, &END_TIME).is_err() );
    }

    #[test]
    fn test_binance_provider_returns_average_price_from_single_time_window() {
        let api_call: AggTradesCall = Box::new( |symbol,_,start_time,end_time,_| {
            assert_eq!( symbol, BinancePriceProvider::SYMBOL.to_string()) ;
            assert_eq!( start_time, Some(START_TIME.timestamp_millis()) );
            assert_eq!( end_time, Some(END_TIME.timestamp_millis()) );
            Ok(FakeBinanceAPI::MULTIPLE_PRICES_RESPONSE.to_string())
        });

        let binance_provider = create_binance_provider_fixture(api_call);
        let prices = binance_provider.prices(&START_TIME, &END_TIME).unwrap();
        assert_eq!( prices.len(), 1 );
        assert_float_absolute_eq!( prices[0], 2.333333333 );
    }

    // #[test]
    // fn test_binance_provider_returns_average_prices_from_multiple_time_windows() {
    //   TODO
    // }


    type AggTradesCall = Box<dyn Fn(&str,Option<i64>,Option<i64>,Option<i64>,Option<i64>) -> anyhow::Result<String> >;
    struct FakeBinanceAPI {
        response: AggTradesCall,
    }
    impl FakeBinanceAPI {
        const SINGLE_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"p": "0.01633102","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;
        const MULTIPLE_PRICES_RESPONSE: &'static str = concat!(
            r#"[{"a": 26129,"p": "1.0","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true },"#,
            r#"{"a": 26129,"p": "2.5","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true },"#,
            r#"{"a": 26129,"p": "3.5","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#
        );
        const MISSING_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;
        const INVALID_PRICE_RESPONSE: &'static str = r#"[{"a": 26129,"p": "notafloat","q": "4.70443515","f": 27781,"l": 27781,"T": 1498793709153,"m": true,"M": true }]"#;
    }
    impl BinanceAPI for FakeBinanceAPI {
        fn agg_trades(&self, 
                      symbol: &str,
                      from_id: Option<i64>,
                      start_time: Option<i64>,
                      end_time: Option<i64>,
                      limit: Option<i64>) -> anyhow::Result<String> {
            (self.response)(symbol,from_id,start_time,end_time,limit)
        }
    }
}
