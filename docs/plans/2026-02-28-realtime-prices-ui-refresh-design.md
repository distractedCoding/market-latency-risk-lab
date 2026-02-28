# Realtime Prices + UI Refresh Design

Date: 2026-02-28
Status: Approved for implementation
Scope: Show per-exchange BTC prices + Polymarket YES pricing in realtime and improve dashboard clarity.

## 1. Intent

Make the dashboard immediately useful for live observation by showing:
- Coinbase BTC/USD
- Binance BTC/USDT
- Kraken XBT/USD
- Polymarket YES bid/ask/mid for the currently tracked market

Keep the existing paper-live flow and PnL cards, but improve the visual hierarchy and readability.

## 2. Architecture

Use server-published snapshots as the single source of truth.

- `lab-server` computes a per-tick snapshot from existing fetched prices and tracked Polymarket quote.
- `api::state` stores the latest snapshot and emits a typed websocket event.
- `api::routes` exposes a REST fallback endpoint for first paint and reconnect recovery.
- `ui` consumes websocket updates primarily and polls REST as fallback.

This avoids browser CORS/rate-limit issues and keeps all values timestamp-aligned per loop tick.

## 3. Data Model

Add a typed `PriceSnapshot` with nullable fields to represent missing data safely:

- `coinbase_btc_usd: Option<f64>`
- `binance_btc_usdt: Option<f64>`
- `kraken_btc_usd: Option<f64>`
- `polymarket_market_id: Option<String>`
- `polymarket_yes_bid: Option<f64>`
- `polymarket_yes_ask: Option<f64>`
- `polymarket_yes_mid: Option<f64>`
- `ts: u64`

Expose via:
- `RuntimeEvent::PriceSnapshot` on websocket stream
- `GET /prices/snapshot` for REST fallback

## 4. UI/UX Direction

- Add a dedicated `Live Prices` panel near the top of the dashboard.
- Show per-venue values in separate rows with directional badges (`up`, `down`, `flat`).
- Display Polymarket market slug and YES bid/ask/mid in the same panel.
- Keep equity chart and portfolio KPIs, but rebalance spacing and typography for scan speed.
- Use subtle update feedback (color pulse) only when values change.

Visual style:
- Maintain the current blue/teal identity but increase contrast and information density.
- Improve panel segmentation and responsive behavior for small screens.

## 5. Realtime Behavior

- Websocket `price_snapshot` updates UI on each live loop tick.
- REST polling (`/prices/snapshot`) runs periodically as a resilience path.
- Staleness indicator marks values stale if no snapshot update arrives for >10 seconds.

## 6. Testing and Verification

- API tests:
  - `GET /prices/snapshot` returns typed payload.
  - websocket forwards `price_snapshot` schema.
- UI tests:
  - HTML shell contains `Live Prices` section and expected labels.
  - JS bundle includes `price_snapshot` routing and `/prices/snapshot` polling.
- Full verification:
  - `PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q`
  - run server and verify panel values update in browser.
