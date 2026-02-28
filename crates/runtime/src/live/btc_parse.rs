use crate::live::btc_feed::NormalizedBtcTick;
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseBtcTradeError {
    InvalidJson,
    UnsupportedMessageType,
    InvalidPrice,
    InvalidSize,
    InvalidTimestamp,
    TimestampOutOfRange,
}

pub fn parse_coinbase_trade(raw: &str) -> Result<NormalizedBtcTick, ParseBtcTradeError> {
    let trade: CoinbaseTrade =
        serde_json::from_str(raw).map_err(|_| ParseBtcTradeError::InvalidJson)?;

    if trade.kind != "match" {
        return Err(ParseBtcTradeError::UnsupportedMessageType);
    }

    let px = trade
        .price
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or(ParseBtcTradeError::InvalidPrice)?;

    let size = trade
        .size
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or(ParseBtcTradeError::InvalidSize)?;

    let ts = parse_timestamp_ms(&trade.time)?;

    Ok(NormalizedBtcTick {
        venue: "coinbase".to_string(),
        px,
        size,
        ts,
    })
}

fn parse_timestamp_ms(timestamp: &str) -> Result<u64, ParseBtcTradeError> {
    let parsed = OffsetDateTime::parse(timestamp, &Rfc3339)
        .map_err(|_| ParseBtcTradeError::InvalidTimestamp)?;
    let unix_millis = parsed.unix_timestamp_nanos() / 1_000_000;
    u64::try_from(unix_millis).map_err(|_| ParseBtcTradeError::TimestampOutOfRange)
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
    use super::{parse_coinbase_trade, ParseBtcTradeError};

    #[test]
    fn parses_coinbase_trade_into_normalized_tick() {
        let raw =
            r#"{"type":"match","price":"64001.2","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let tick = parse_coinbase_trade(raw).unwrap();
        assert_eq!(tick.venue, "coinbase");
        assert!(tick.px > 0.0);
        assert_eq!(tick.ts, 1_772_280_000_000);
    }

    #[test]
    fn rejects_coinbase_message_types_other_than_match() {
        let raw =
            r#"{"type":"ticker","price":"64001.2","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::UnsupportedMessageType);
    }

    #[test]
    fn rejects_coinbase_trades_with_invalid_price() {
        let raw = r#"{"type":"match","price":"oops","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::InvalidPrice);
    }

    #[test]
    fn rejects_coinbase_trades_with_invalid_timestamp() {
        let raw = r#"{"type":"match","price":"64001.2","size":"0.01","time":"not-a-timestamp"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::InvalidTimestamp);
    }

    #[test]
    fn rejects_coinbase_trades_with_pre_epoch_timestamp() {
        let raw =
            r#"{"type":"match","price":"64001.2","size":"0.01","time":"1969-12-31T23:59:59Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::TimestampOutOfRange);
    }

    #[test]
    fn rejects_coinbase_trades_with_non_finite_price() {
        let raw = r#"{"type":"match","price":"inf","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
        let error = parse_coinbase_trade(raw).unwrap_err();

        assert_eq!(error, ParseBtcTradeError::InvalidPrice);
    }
}
