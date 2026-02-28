use crate::live::btc_feed::NormalizedBtcTick;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseBtcTradeError {
    InvalidJson,
    UnsupportedMessageType,
    InvalidPrice,
    InvalidSize,
}

pub fn parse_coinbase_trade(raw: &str) -> Result<NormalizedBtcTick, ParseBtcTradeError> {
    let trade: CoinbaseTrade = serde_json::from_str(raw).map_err(|_| ParseBtcTradeError::InvalidJson)?;

    if trade.kind != "match" {
        return Err(ParseBtcTradeError::UnsupportedMessageType);
    }

    let px = trade
        .price
        .parse::<f64>()
        .ok()
        .filter(|value| *value > 0.0)
        .ok_or(ParseBtcTradeError::InvalidPrice)?;

    let size = trade
        .size
        .parse::<f64>()
        .ok()
        .filter(|value| *value > 0.0)
        .ok_or(ParseBtcTradeError::InvalidSize)?;

    Ok(NormalizedBtcTick {
        venue: "coinbase".to_string(),
        px,
        size,
        ts: trade.time,
    })
}

#[derive(Debug, Deserialize)]
struct CoinbaseTrade {
    #[serde(rename = "type")]
    kind: String,
    price: String,
    size: String,
    time: String,
}

#[cfg(test)]
mod tests {
    use super::{ParseBtcTradeError, parse_coinbase_trade};

    #[test]
    fn parses_coinbase_trade_into_normalized_tick() {
        let raw = r#"{"type":"match","price":"64001.2","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let tick = parse_coinbase_trade(raw).unwrap();
        assert_eq!(tick.venue, "coinbase");
        assert!(tick.px > 0.0);
    }

    #[test]
    fn rejects_coinbase_message_types_other_than_match() {
        let raw = r#"{"type":"ticker","price":"64001.2","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::UnsupportedMessageType);
    }

    #[test]
    fn rejects_coinbase_trades_with_invalid_price() {
        let raw = r#"{"type":"match","price":"oops","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::InvalidPrice);
    }
}
