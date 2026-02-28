use crate::events::{RuntimeEvent, RuntimeStage};
use crate::live::{BtcMedianTick, PolymarketQuoteTick};
use crate::paper_exec::{paper_fill_buy, paper_fill_sell};
use strategy::{live_signal, RiskState, Signal};

#[derive(Debug, Clone)]
pub struct JoinedLiveInputs {
    pub btc_tick: BtcMedianTick,
    pub quote_tick: PolymarketQuoteTick,
}

const BTC_SPREAD_TO_PRICE_COEFF: f64 = 0.001;
const SIGNAL_THRESHOLD: f64 = 0.01;
const ORDER_QTY: f64 = 1.0;
const ORDER_SLIPPAGE_BPS: f64 = 0.0;
const ORDER_FEE_BPS: f64 = 0.0;
const RISK_STARTING_EQUITY: f64 = 10.0;
const RISK_DAILY_LOSS_CAP_PCT: f64 = 0.06;
const SELL_BASE_MARKET_EXPOSURE: f64 = 1.0;

pub fn run_paper_live_once(tick: u64, joined: &JoinedLiveInputs) -> Vec<RuntimeEvent> {
    let prediction_price =
        derive_prediction_price(joined.quote_tick.mid_yes, joined.btc_tick.px_spread);
    let live_signal = match live_signal(
        prediction_price,
        joined.quote_tick.mid_yes,
        SIGNAL_THRESHOLD,
    ) {
        Ok(signal) => signal,
        Err(_) => return vec![],
    };

    if live_signal.action == Signal::Hold {
        return vec![];
    }

    let mut events = vec![RuntimeEvent::new(tick, RuntimeStage::PaperIntentCreated)];
    let signed_exposure_delta =
        signed_exposure_delta(live_signal.action, ORDER_QTY, joined.quote_tick.mid_yes);
    let current_market_exposure = current_market_exposure(live_signal.action);

    let risk_state = match RiskState::new(RISK_STARTING_EQUITY, RISK_DAILY_LOSS_CAP_PCT) {
        Ok(state) => state,
        Err(_) => return events,
    };

    if risk_state
        .check_market_exposure(
            &joined.quote_tick.market_slug,
            current_market_exposure,
            signed_exposure_delta,
        )
        .is_err()
    {
        return events;
    }

    let fill_result = match live_signal.action {
        Signal::Buy => paper_fill_buy(
            joined.quote_tick.best_yes_ask,
            ORDER_QTY,
            ORDER_SLIPPAGE_BPS,
            ORDER_FEE_BPS,
        ),
        Signal::Sell => paper_fill_sell(
            joined.quote_tick.best_yes_bid,
            ORDER_QTY,
            ORDER_SLIPPAGE_BPS,
            ORDER_FEE_BPS,
        ),
        Signal::Hold => return vec![],
    };

    if fill_result.is_ok() {
        events.push(RuntimeEvent::new(tick, RuntimeStage::PaperFillRecorded));
    }

    events
}

fn derive_prediction_price(mid_yes: f64, btc_spread_signal: f64) -> f64 {
    (mid_yes + (btc_spread_signal * BTC_SPREAD_TO_PRICE_COEFF)).clamp(0.0, 1.0)
}

fn signed_exposure_delta(action: Signal, qty: f64, reference_yes_price: f64) -> f64 {
    let unsigned_notional = qty * reference_yes_price;

    match action {
        Signal::Buy => unsigned_notional,
        Signal::Sell => -unsigned_notional,
        Signal::Hold => 0.0,
    }
}

fn current_market_exposure(action: Signal) -> f64 {
    match action {
        Signal::Buy => 0.0,
        Signal::Sell => SELL_BASE_MARKET_EXPOSURE,
        Signal::Hold => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::{run_paper_live_once, JoinedLiveInputs};
    use crate::events::RuntimeStage;
    use crate::live::{BtcMedianTick, PolymarketQuoteTick};

    #[test]
    fn run_paper_live_once_emits_intent_then_fill_for_buy_signal() {
        let out = run_paper_live_once(42, &joined_inputs_for_buy_signal(42));

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].stage, RuntimeStage::PaperIntentCreated);
        assert_eq!(out[1].stage, RuntimeStage::PaperFillRecorded);
    }

    #[test]
    fn run_paper_live_once_emits_no_events_for_hold_signal() {
        let out = run_paper_live_once(42, &joined_inputs_for_hold_signal(42));

        assert!(out.is_empty());
    }

    #[test]
    fn run_paper_live_once_emits_only_intent_when_risk_rejects() {
        let out = run_paper_live_once(42, &joined_inputs_for_risk_rejected_buy(42));

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].stage, RuntimeStage::PaperIntentCreated);
    }

    #[test]
    fn run_paper_live_once_emits_no_events_for_invalid_signal_input() {
        let out = run_paper_live_once(42, &joined_inputs_with_zero_mid_price(42));

        assert!(out.is_empty());
    }

    fn joined_inputs_for_buy_signal(tick: u64) -> JoinedLiveInputs {
        JoinedLiveInputs {
            btc_tick: BtcMedianTick::new(64_000.0, 8.0, 3, tick),
            quote_tick: PolymarketQuoteTick {
                market_slug: "btc-up-down".to_string(),
                best_yes_bid: 0.48,
                best_yes_ask: 0.52,
                mid_yes: 0.50,
                ts: tick,
            },
        }
    }

    fn joined_inputs_for_hold_signal(tick: u64) -> JoinedLiveInputs {
        JoinedLiveInputs {
            btc_tick: BtcMedianTick::new(64_000.0, 0.0, 3, tick),
            quote_tick: PolymarketQuoteTick {
                market_slug: "btc-up-down".to_string(),
                best_yes_bid: 0.48,
                best_yes_ask: 0.52,
                mid_yes: 0.50,
                ts: tick,
            },
        }
    }

    fn joined_inputs_for_risk_rejected_buy(tick: u64) -> JoinedLiveInputs {
        JoinedLiveInputs {
            btc_tick: BtcMedianTick::new(64_000.0, 12.0, 3, tick),
            quote_tick: PolymarketQuoteTick {
                market_slug: "btc-up-down".to_string(),
                best_yes_bid: 0.89,
                best_yes_ask: 0.91,
                mid_yes: 0.90,
                ts: tick,
            },
        }
    }

    fn joined_inputs_with_zero_mid_price(tick: u64) -> JoinedLiveInputs {
        JoinedLiveInputs {
            btc_tick: BtcMedianTick::new(64_000.0, 8.0, 3, tick),
            quote_tick: PolymarketQuoteTick {
                market_slug: "btc-up-down".to_string(),
                best_yes_bid: 0.0,
                best_yes_ask: 0.0,
                mid_yes: 0.0,
                ts: tick,
            },
        }
    }
}
