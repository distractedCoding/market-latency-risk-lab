# Dashboard Settings + BTC 15m Forecast Design

Date: 2026-02-28
Status: Approved for implementation

## Goal

Add an operator dashboard that lets users change core trading settings in-app, shows key account/strategy metrics at the top, renders the main analytics in the center, and presents execution logs in a chat-style stream on the right.

## Layout

- Top KPI strip (always visible on desktop):
  - Balance
  - Total P&L
  - Exec Latency
  - Win Rate
  - BTC/USD Price
- Main content (desktop):
  - Left rail: settings controls
  - Middle: dashboard content (equity + BTC 15m forecast)
  - Right rail: chat-style execution logs
- Mobile:
  - Stack as KPI strip -> settings -> middle dashboard -> execution logs

## Product Scope Locks

- Market display and forecast scope are locked to BTC.
- Forecast horizon is fixed to 15 minutes.
- Core editable settings in UI:
  - execution mode
  - trading pause/resume
  - lag threshold
  - risk per trade
  - daily loss cap

## Backend Model

- Add a server-authoritative settings snapshot in API state.
- Add strategy KPI snapshot for top strip values.
- Add BTC 15m forecast snapshot payload.
- Add normalized execution log entries for chat-style UI rendering.

## API and WebSocket

- REST:
  - `GET /settings`
  - `PATCH /settings`
  - `GET /strategy/stats`
  - `GET /forecast/btc-15m`
- WebSocket events:
  - `settings_updated`
  - `strategy_stats`
  - `btc_forecast`
  - `execution_log`

REST is first paint/reconnect fallback; websocket is primary realtime channel.

## Runtime Rules

- Runtime loop reads settings from state each tick.
- `trading_paused=true` blocks new intents/fills but keeps telemetry flowing.
- `execution_mode=live` is rejected unless live feature gate is enabled.
- Daily cap and per-trade risk remain enforced server-side.

## KPI Definitions

- Balance: current equity.
- Total P&L: equity minus session starting equity.
- Exec Latency: latest decision/execution latency sample in microseconds.
- Win Rate: winning closed trades / total closed trades.
- BTC/USD: current consolidated mark from venue median.

## Forecast Definition

- Horizon fixed at 15m.
- Payload includes:
  - `current_btc_usd`
  - `forecast_btc_usd`
  - `delta_pct`
  - `horizon_minutes=15`
  - `ts`

## Testing

- API tests for new routes and event payloads.
- Validation tests for settings patch constraints.
- UI tests for required KPI labels and layout regions.
- JS tests for new event routing and settings PATCH behavior.
