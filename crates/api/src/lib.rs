pub mod routes;
pub mod state;
pub mod ws;

use axum::Router;

pub fn module_ready() -> bool {
    true
}

pub fn app() -> Router {
    routes::router(state::AppState::new())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{header, Request, StatusCode},
    };
    use futures_util::StreamExt;
    use serde::Deserialize;
    use serde_json::Value;
    use std::time::Duration;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;
    use tower::ServiceExt;

    use crate::{
        app, routes,
        state::{
            AppState, DiscoveredMarket as StateDiscoveredMarket, FeedMode, PaperOrderSide,
            RuntimeEvent, SourceCount as StateSourceCount,
        },
    };

    #[derive(Debug, Deserialize)]
    struct StartRunResponse {
        run_id: u64,
    }

    #[derive(Debug)]
    struct StartRunResult {
        status: StatusCode,
        location: Option<String>,
        payload: StartRunResponse,
    }

    #[derive(Debug, Deserialize)]
    struct SourceCount {
        source: String,
        count: u64,
    }

    #[derive(Debug, Deserialize)]
    struct FeedHealthResponse {
        mode: FeedMode,
        source_counts: Vec<SourceCount>,
    }

    #[derive(Debug, Deserialize)]
    struct DiscoveredMarket {
        source: String,
        market_id: String,
    }

    #[derive(Debug, Deserialize)]
    struct DiscoveredMarketsResponse {
        markets: Vec<DiscoveredMarket>,
    }

    #[derive(Debug, Deserialize)]
    struct PriceSnapshotResponse {
        coinbase_btc_usd: Option<f64>,
        binance_btc_usdt: Option<f64>,
        kraken_btc_usd: Option<f64>,
        polymarket_market_id: Option<String>,
        polymarket_yes_bid: Option<f64>,
        polymarket_yes_ask: Option<f64>,
        polymarket_yes_mid: Option<f64>,
        ts: u64,
    }

    #[derive(Debug, Deserialize)]
    struct StrategyPerfResponse {
        execution_mode: String,
        lag_threshold_pct: f64,
        decision_p95_us: u64,
        intents_per_sec: u64,
        fills_per_sec: u64,
        lag_triggers: u64,
        halted: bool,
    }

    async fn start_run_request(app: axum::Router) -> StartRunResult {
        let response = app
            .oneshot(Request::post("/runs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: StartRunResponse = serde_json::from_slice(&body).unwrap();

        StartRunResult {
            status,
            location,
            payload,
        }
    }

    async fn send_get(app: &axum::Router, path: &str) -> axum::response::Response {
        app.clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn send_patch_json(
        app: &axum::Router,
        path: &str,
        payload: Value,
    ) -> axum::response::Response {
        app.clone()
            .oneshot(
                Request::patch(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn parse_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn next_ws_json() -> Value {
        next_ws_json_for_event(RuntimeEvent::paper_fill(
            "btc-up-down",
            PaperOrderSide::Buy,
            5.0,
            0.52,
        ))
        .await
    }

    async fn next_ws_json_for_event(event: RuntimeEvent) -> Value {
        let state = AppState::new();
        let app = routes::router(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{addr}/ws/events");
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();

        let _ = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        state.publish_event(event).unwrap();

        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        server.abort();

        match message {
            Message::Text(text) => serde_json::from_str(text.as_ref()).unwrap(),
            other => panic!("expected text websocket message, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn post_runs_returns_run_id_and_location() {
        let app = app();

        let result = start_run_request(app).await;

        assert_eq!(result.status, StatusCode::CREATED);
        assert_eq!(result.payload.run_id, 1);
        assert_eq!(result.location.as_deref(), Some("/runs/1"));
    }

    #[tokio::test]
    async fn post_runs_returns_monotonic_unique_run_ids() {
        let app = app();

        let result_one = start_run_request(app.clone()).await;
        let result_two = start_run_request(app.clone()).await;
        let result_three = start_run_request(app).await;

        assert_eq!(result_one.status, StatusCode::CREATED);
        assert_eq!(result_two.status, StatusCode::CREATED);
        assert_eq!(result_three.status, StatusCode::CREATED);

        assert_eq!(result_one.payload.run_id, 1);
        assert_eq!(result_two.payload.run_id, 2);
        assert_eq!(result_three.payload.run_id, 3);
        assert_eq!(result_one.location.as_deref(), Some("/runs/1"));
        assert_eq!(result_two.location.as_deref(), Some("/runs/2"));
        assert_eq!(result_three.location.as_deref(), Some("/runs/3"));
    }

    #[tokio::test]
    async fn get_feed_health_returns_mode_and_source_counts() {
        let app = app();
        let res = send_get(&app, "/feed/health").await;
        assert_eq!(res.status(), 200);
    }

    #[tokio::test]
    async fn get_feed_health_returns_typed_payload() {
        let app = routes::router(AppState::with_feed_data_for_test(
            FeedMode::Sim,
            vec![
                StateSourceCount {
                    source: "polymarket".to_owned(),
                    count: 12,
                },
                StateSourceCount {
                    source: "kalshi".to_owned(),
                    count: 4,
                },
            ],
            vec![StateDiscoveredMarket {
                source: "polymarket".to_owned(),
                market_id: "btc-up-down".to_owned(),
            }],
        ));
        let response = send_get(&app, "/feed/health").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: FeedHealthResponse = parse_json(response).await;
        assert_eq!(payload.mode, FeedMode::Sim);
        assert_eq!(payload.source_counts.len(), 2);
        assert_eq!(payload.source_counts[0].source, "polymarket");
        assert_eq!(payload.source_counts[0].count, 12);
        assert_eq!(payload.source_counts[1].source, "kalshi");
        assert_eq!(payload.source_counts[1].count, 4);
    }

    #[tokio::test]
    async fn get_markets_discovered_returns_typed_payload() {
        let app = routes::router(AppState::with_feed_data_for_test(
            FeedMode::PaperLive,
            vec![StateSourceCount {
                source: "polymarket".to_owned(),
                count: 3,
            }],
            vec![
                StateDiscoveredMarket {
                    source: "polymarket".to_owned(),
                    market_id: "btc-up-down".to_owned(),
                },
                StateDiscoveredMarket {
                    source: "polymarket".to_owned(),
                    market_id: "eth-up-down".to_owned(),
                },
            ],
        ));
        let response = send_get(&app, "/markets/discovered").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: DiscoveredMarketsResponse = parse_json(response).await;
        assert_eq!(payload.markets.len(), 2);
        assert_eq!(payload.markets[0].source, "polymarket");
        assert_eq!(payload.markets[0].market_id, "btc-up-down");
        assert_eq!(payload.markets[1].source, "polymarket");
        assert_eq!(payload.markets[1].market_id, "eth-up-down");
    }

    #[tokio::test]
    async fn get_portfolio_summary_returns_typed_payload() {
        let state = AppState::new();
        state.set_portfolio_summary(crate::state::PortfolioSummary {
            equity: 123.45,
            pnl: 23.45,
            position_qty: 7.0,
            fills: 42,
        });
        let app = routes::router(state);

        let response = send_get(&app, "/portfolio/summary").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = parse_json(response).await;
        assert_eq!(payload["equity"].as_f64(), Some(123.45));
        assert_eq!(payload["pnl"].as_f64(), Some(23.45));
        assert_eq!(payload["position_qty"].as_f64(), Some(7.0));
        assert_eq!(payload["fills"].as_u64(), Some(42));
    }

    #[tokio::test]
    async fn get_prices_snapshot_returns_typed_payload() {
        let state = AppState::new();
        state.set_price_snapshot(crate::state::PriceSnapshot {
            coinbase_btc_usd: Some(64_101.2),
            binance_btc_usdt: Some(64_100.9),
            kraken_btc_usd: Some(64_101.0),
            polymarket_market_id: Some("btc-up-down".to_owned()),
            polymarket_yes_bid: Some(0.481),
            polymarket_yes_ask: Some(0.487),
            polymarket_yes_mid: Some(0.484),
            ts: 77,
        });
        let app = routes::router(state);

        let response = send_get(&app, "/prices/snapshot").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: PriceSnapshotResponse = parse_json(response).await;
        assert_eq!(payload.coinbase_btc_usd, Some(64_101.2));
        assert_eq!(payload.binance_btc_usdt, Some(64_100.9));
        assert_eq!(payload.kraken_btc_usd, Some(64_101.0));
        assert_eq!(payload.polymarket_market_id.as_deref(), Some("btc-up-down"));
        assert_eq!(payload.polymarket_yes_bid, Some(0.481));
        assert_eq!(payload.polymarket_yes_ask, Some(0.487));
        assert_eq!(payload.polymarket_yes_mid, Some(0.484));
        assert_eq!(payload.ts, 77);
    }

    #[tokio::test]
    async fn get_strategy_perf_returns_latency_and_throughput() {
        let state = AppState::new();
        state.set_strategy_perf_summary(crate::state::StrategyPerfSummary {
            execution_mode: "paper".to_owned(),
            lag_threshold_pct: 0.3,
            decision_p95_us: 84,
            intents_per_sec: 1200,
            fills_per_sec: 840,
            lag_triggers: 15,
            halted: false,
        });
        let app = routes::router(state);

        let response = send_get(&app, "/strategy/perf").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: StrategyPerfResponse = parse_json(response).await;
        assert_eq!(payload.execution_mode, "paper");
        assert_eq!(payload.lag_threshold_pct, 0.3);
        assert_eq!(payload.decision_p95_us, 84);
        assert_eq!(payload.intents_per_sec, 1200);
        assert_eq!(payload.fills_per_sec, 840);
        assert_eq!(payload.lag_triggers, 15);
        assert!(!payload.halted);
    }

    #[tokio::test]
    async fn get_settings_returns_runtime_controls() {
        let app = app();

        let response = send_get(&app, "/settings").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = parse_json(response).await;
        assert_eq!(payload["execution_mode"], "paper");
        assert_eq!(payload["market"], "BTC/USD");
        assert_eq!(payload["forecast_horizon_minutes"], 15);
    }

    #[tokio::test]
    async fn patch_settings_updates_runtime_controls() {
        let app = app();

        let response = send_patch_json(
            &app,
            "/settings",
            serde_json::json!({
                "trading_paused": true,
                "lag_threshold_pct": 0.45,
                "risk_per_trade_pct": 0.6,
                "daily_loss_cap_pct": 2.5
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = parse_json(response).await;
        assert_eq!(payload["trading_paused"], true);
        assert_eq!(payload["lag_threshold_pct"].as_f64(), Some(0.45));
        assert_eq!(payload["risk_per_trade_pct"].as_f64(), Some(0.6));
        assert_eq!(payload["daily_loss_cap_pct"].as_f64(), Some(2.5));
    }

    #[tokio::test]
    async fn patch_settings_rejects_live_mode_when_feature_disabled() {
        let app = app();

        let response = send_patch_json(
            &app,
            "/settings",
            serde_json::json!({
                "execution_mode": "live"
            }),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_strategy_stats_returns_top_kpis() {
        let app = app();

        let response = send_get(&app, "/strategy/stats").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = parse_json(response).await;
        assert!(payload.get("balance").is_some());
        assert!(payload.get("total_pnl").is_some());
        assert!(payload.get("exec_latency_us").is_some());
        assert!(payload.get("win_rate").is_some());
        assert!(payload.get("btc_usd").is_some());
    }

    #[tokio::test]
    async fn get_btc_15m_forecast_returns_fixed_horizon_payload() {
        let app = app();

        let response = send_get(&app, "/forecast/btc-15m").await;

        assert_eq!(response.status(), StatusCode::OK);
        let payload: Value = parse_json(response).await;
        assert_eq!(payload["horizon_minutes"], 15);
        assert!(payload.get("current_btc_usd").is_some());
        assert!(payload.get("forecast_btc_usd").is_some());
        assert!(payload.get("delta_pct").is_some());
    }

    #[tokio::test]
    async fn post_runs_returns_internal_server_error_on_run_id_overflow() {
        let app = routes::router(AppState::with_next_run_id_for_test(u64::MAX));

        let response = app
            .oneshot(Request::post("/runs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_root_serves_dashboard_shell() {
        let app = app();

        let response = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        assert_eq!(content_type, Some("text/html; charset=utf-8"));

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("/static/styles.css"));
        assert!(html.contains("/static/app.js"));
        assert!(html.contains("/ws/events"));
    }

    #[tokio::test]
    async fn get_static_assets_serves_css_and_js() {
        let app = app();

        let css_response = app
            .clone()
            .oneshot(
                Request::get("/static/styles.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(css_response.status(), StatusCode::OK);
        let css_type = css_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        assert_eq!(css_type, Some("text/css; charset=utf-8"));

        let js_response = app
            .oneshot(Request::get("/static/app.js").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(js_response.status(), StatusCode::OK);
        let js_type = js_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        assert_eq!(js_type, Some("application/javascript; charset=utf-8"));
    }

    #[tokio::test]
    async fn websocket_streams_events_channel() {
        let state = AppState::new();
        let app = routes::router(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{addr}/ws/events");
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        let payload = match message {
            Message::Text(text) => text,
            other => panic!("expected text websocket message, got {other:?}"),
        };
        let value: Value = serde_json::from_str(payload.as_ref()).unwrap();
        assert_eq!(
            value.get("event_type").and_then(Value::as_str),
            Some("connected")
        );
        assert_eq!(value.get("run_id"), Some(&Value::Null));

        server.abort();
    }

    #[tokio::test]
    async fn websocket_forwards_published_events() {
        let state = AppState::new();
        let app = routes::router(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{addr}/ws/events");
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();

        let _ = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        state.publish_event(RuntimeEvent::run_started(42)).unwrap();

        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let payload = match message {
            Message::Text(text) => text,
            other => panic!("expected text websocket message, got {other:?}"),
        };
        let value: Value = serde_json::from_str(payload.as_ref()).unwrap();

        assert_eq!(
            value.get("event_type").and_then(Value::as_str),
            Some("run_started")
        );
        assert_eq!(value.get("run_id").and_then(Value::as_u64), Some(42));

        server.abort();
    }

    #[tokio::test]
    async fn websocket_emits_paper_fill_event_payload() {
        let msg = next_ws_json().await;
        assert_eq!(msg["event_type"], "paper_fill");
        assert_eq!(msg["market_id"], "btc-up-down");
        assert_eq!(msg["side"], "buy");
        assert_eq!(msg["qty"].as_f64(), Some(5.0));
        assert_eq!(msg["fill_px"].as_f64(), Some(0.52));
    }

    #[tokio::test]
    async fn websocket_emits_paper_intent_event_payload() {
        let msg = next_ws_json_for_event(RuntimeEvent::paper_intent(
            "btc-up-down",
            PaperOrderSide::Sell,
            3.0,
            0.49,
        ))
        .await;

        assert_eq!(msg["event_type"], "paper_intent");
        assert_eq!(msg["market_id"], "btc-up-down");
        assert_eq!(msg["side"], "sell");
        assert!(msg["qty"].as_f64().is_some());
        assert!(msg["limit_px"].as_f64().is_some());
    }

    #[tokio::test]
    async fn websocket_emits_risk_reject_event_payload() {
        let msg = next_ws_json_for_event(RuntimeEvent::risk_reject(
            "btc-up-down",
            "max_market_exposure",
            7.0,
        ))
        .await;

        assert_eq!(msg["event_type"], "risk_reject");
        assert_eq!(msg["market_id"], "btc-up-down");
        assert_eq!(msg["reason"], "max_market_exposure");
        assert!(msg["requested_qty"].as_f64().is_some());
    }

    #[tokio::test]
    async fn websocket_emits_price_snapshot_event_payload() {
        let msg =
            next_ws_json_for_event(RuntimeEvent::price_snapshot(crate::state::PriceSnapshot {
                coinbase_btc_usd: Some(64_122.3),
                binance_btc_usdt: Some(64_121.9),
                kraken_btc_usd: Some(64_122.1),
                polymarket_market_id: Some("btc-march".to_owned()),
                polymarket_yes_bid: Some(0.49),
                polymarket_yes_ask: Some(0.51),
                polymarket_yes_mid: Some(0.50),
                ts: 901,
            }))
            .await;

        assert_eq!(msg["event_type"], "price_snapshot");
        assert_eq!(msg["coinbase_btc_usd"].as_f64(), Some(64_122.3));
        assert_eq!(msg["binance_btc_usdt"].as_f64(), Some(64_121.9));
        assert_eq!(msg["kraken_btc_usd"].as_f64(), Some(64_122.1));
        assert_eq!(msg["polymarket_market_id"], "btc-march");
        assert_eq!(msg["polymarket_yes_bid"].as_f64(), Some(0.49));
        assert_eq!(msg["polymarket_yes_ask"].as_f64(), Some(0.51));
        assert_eq!(msg["polymarket_yes_mid"].as_f64(), Some(0.50));
        assert_eq!(msg["ts"].as_u64(), Some(901));
    }

    #[tokio::test]
    async fn websocket_emits_strategy_perf_event_payload() {
        let msg = next_ws_json_for_event(RuntimeEvent::strategy_perf(
            crate::state::StrategyPerfSummary {
                execution_mode: "paper".to_owned(),
                lag_threshold_pct: 0.3,
                decision_p95_us: 76,
                intents_per_sec: 1400,
                fills_per_sec: 990,
                lag_triggers: 22,
                halted: false,
            },
        ))
        .await;

        assert_eq!(msg["event_type"], "strategy_perf");
        assert_eq!(msg["execution_mode"], "paper");
        assert_eq!(msg["lag_threshold_pct"].as_f64(), Some(0.3));
        assert_eq!(msg["decision_p95_us"].as_u64(), Some(76));
        assert_eq!(msg["intents_per_sec"].as_u64(), Some(1400));
        assert_eq!(msg["fills_per_sec"].as_u64(), Some(990));
        assert_eq!(msg["lag_triggers"].as_u64(), Some(22));
        assert_eq!(msg["halted"].as_bool(), Some(false));
    }
}
