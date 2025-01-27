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