use crate::events::{RuntimeEvent, RuntimeStage};
use crate::live::{BtcMedianTick, PolymarketQuoteTick};

#[derive(Debug, Clone)]
pub struct JoinedLiveInputs {
    pub btc_tick: BtcMedianTick,
    pub quote_tick: PolymarketQuoteTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    Filled,
    Unfilled,
}

pub fn run_paper_live_once(tick: u64, joined: &JoinedLiveInputs) -> Vec<RuntimeEvent> {
    let execution = deterministic_execution_outcome(joined);
    events_for_execution_outcome(tick, execution)
}

fn deterministic_execution_outcome(joined: &JoinedLiveInputs) -> ExecutionResult {
    let _is_btc_trending_up = joined.btc_tick.px_spread >= 0.0;
    let _reference_yes_price = joined.quote_tick.mid_yes;

    ExecutionResult::Filled
}

fn events_for_execution_outcome(tick: u64, execution: ExecutionResult) -> Vec<RuntimeEvent> {
    let mut events = vec![RuntimeEvent::new(tick, RuntimeStage::PaperIntentCreated)];

    if execution == ExecutionResult::Filled {
        events.push(RuntimeEvent::new(tick, RuntimeStage::PaperFillRecorded));
    }

    events
}

#[cfg(test)]
mod tests {
    use super::{ExecutionResult, JoinedLiveInputs, events_for_execution_outcome, run_paper_live_once};
    use crate::events::RuntimeStage;
    use crate::live::{BtcMedianTick, PolymarketQuoteTick};

    #[test]
    fn run_paper_live_once_emits_fill_for_deterministic_stub() {
        let out = run_paper_live_once(42, &synthetic_joined_live_inputs(42));

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].stage, RuntimeStage::PaperIntentCreated);
        assert_eq!(out[1].stage, RuntimeStage::PaperFillRecorded);
    }

    #[test]
    fn unfilled_outcome_emits_only_intent_event() {
        let out = events_for_execution_outcome(42, ExecutionResult::Unfilled);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].stage, RuntimeStage::PaperIntentCreated);
    }

    fn synthetic_joined_live_inputs(tick: u64) -> JoinedLiveInputs {
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
}
