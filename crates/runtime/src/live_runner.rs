use crate::events::{RuntimeEvent, RuntimeStage};
use crate::live::{BtcMedianTick, PolymarketQuoteTick};

#[derive(Debug, Clone)]
struct JoinedLiveInputs {
    btc_tick: BtcMedianTick,
    quote_tick: PolymarketQuoteTick,
}

pub fn run_paper_live_once(tick: u64) -> Vec<RuntimeEvent> {
    let joined = synthetic_joined_live_inputs(tick);

    let _is_btc_trending_up = joined.btc_tick.px_spread >= 0.0;
    let _reference_yes_price = joined.quote_tick.mid_yes;

    vec![
        RuntimeEvent::new(tick, RuntimeStage::PaperIntentCreated),
        RuntimeEvent::new(tick, RuntimeStage::PaperFillRecorded),
    ]
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
