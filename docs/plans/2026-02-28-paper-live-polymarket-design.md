# Paper-Live Polymarket Adapter Design

Date: 2026-02-28
Status: Approved for implementation planning
Scope: Live-data paper-trading mode with auto strategy, no real order routing

## 1. Intent

Add a paper-trading mode that ingests live BTC reference prices and live Polymarket market data, auto-discovers markets, and runs an automatic strategy that generates paper orders and paper fills.

This mode is default in v1 (`paper_live`) and keeps strict no-live-execution boundaries.

## 2. Product Decisions (Locked)

- BTC reference feed: multi-venue median.
- Polymarket market scope: auto-discovery.
- Fill model: Polymarket BBO plus configurable slippage and fees.
- Operating mode: automatic paper strategy (not manual only).
- Default runtime profile: paper mode by default, simulation available as opt-out fallback.

## 3. Architecture

Use the existing Rust monolith and add a live ingestion plane in-process.

### Core planes

- Live ingest plane (`runtime`):
  - BTC venue workers (for example Coinbase/Binance/Kraken) normalize ticks.
  - BTC aggregator computes robust median and quality metrics.
  - Polymarket discovery worker finds markets by filter policy and subscribes to quote streams.
- Strategy/risk plane (`strategy`):
  - Computes divergence from BTC reference versus Polymarket quote context.
  - Emits paper intents subject to risk and cooldown constraints.
- Paper execution plane (`runtime`):
  - Applies BBO + slippage + fee model to intents.
  - Produces paper fills, positions, and PnL updates.
- Interface plane (`api` + `ui`):
  - HTTP and WebSocket endpoints for health, market discovery, intents, fills, and risk telemetry.

## 4. Data Flow

1. BTC workers emit `BtcTick { venue, px, ts }`.
2. Aggregator emits `BtcMedianTick { px_median, px_spread, venue_count, ts }`.
3. Polymarket worker emits `PolyQuoteTick { market_id, bid, ask, mid, ts }`.
4. Join stage pairs latest valid BTC median with each market quote.
5. Strategy computes divergence score and emits `PaperIntent` when threshold/risk criteria pass.
6. Paper execution computes fill at BBO plus slippage/fees and emits `PaperFill`.
7. Portfolio/risk updates produce PnL, exposure, rejection events, and UI snapshots.

## 5. Paper Execution Semantics

- Buy fill basis: ask side.
- Sell fill basis: bid side.
- Slippage: applied in basis points from BBO.
- Fees: applied via configurable bps model.
- v1 fill simplification: full-fill-per-intent allowed, with max notional and max position constraints.

Formula examples:

- Buy: `fill_px = ask * (1 + slippage_bps / 10_000) + fee_component`
- Sell: `fill_px = bid * (1 - slippage_bps / 10_000) - fee_component`

## 6. Risk and Safety Boundaries

- Hard no-live-order boundary: no authenticated execution routes or private order submission paths.
- Risk guards:
  - per-market max exposure
  - global max leverage/notional
  - daily drawdown cap
  - cooldown throttles after consecutive losses/rejections
- Feed-quality guards:
  - stale source detection
  - low venue-count detection
  - cross-venue spread sanity checks

If quality drops below thresholds, strategy auto-pauses and emits explicit operator-visible events.

## 7. Observability and Replay

- Persist normalized feed snapshots and decision traces for replay.
- Continue replay artifact generation with compatible CSV headers.
- Add paper-trade journal rows (intents, fills, rejects, risk halts).
- WS channels expose: market discovery, feed health, signals, intents, fills, and latency snapshots.

## 8. API and UI Implications

- API additions:
  - current mode and feed health status
  - discovered markets view
  - paper positions, fills, and risk rejections
- UI additions:
  - live discovered-market panel
  - BTC median quality panel (venue count, spread)
  - paper order/fill tape
  - PnL and exposure panel
  - mode banner (`paper_live` vs `sim`)

## 9. Failure Handling

- Supervisor restarts per-feed workers with exponential backoff and jitter.
- Parse errors are counted and surfaced; malformed payload bursts can trip temporary source suppression.
- WS disconnects are non-fatal and reconnect safely.
- Any critical invariant breach triggers paper trading halt with reason code.

## 10. Testing Strategy

- Unit tests:
  - median aggregation, outlier clipping, stale-source exclusion
  - divergence scoring
  - BBO + slippage + fee math
  - risk gate outcomes
- Integration tests with mocked streams:
  - market auto-discovery and subscription lifecycle
  - intent generation and fill lifecycle
  - reconnect and stale-feed handling
- Contract tests:
  - API and WS payload schema stability for dashboard consumers

## 11. Documentation and Positioning Changes

- Update repository docs from "simulation-only" to:
  - live-data paper-trading supported by default
  - still no real-money execution or live order routing
- Add operator startup checklist for required env vars and expected healthy-state signals.

## 12. Non-Goals for This Phase

- No private key management for real execution.
- No true matching-engine microstructure emulation beyond paper model.
- No production deployment hardening beyond local/dev reliability requirements.
