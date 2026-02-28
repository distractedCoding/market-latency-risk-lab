use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    state::{
        AppState, BtcForecastSummary, DiscoveredMarketsResponse, ExecutionLogEntry,
        FeedHealthResponse, PortfolioSummary, PriceSnapshot, RuntimeEvent, RuntimeSettings,
        RuntimeSettingsPatch, StrategyPerfSummary, StrategyStatsSummary,
    },
    ws,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard_index))
        .route("/feed/health", get(feed_health))
        .route("/markets/discovered", get(markets_discovered))
        .route("/prices/snapshot", get(prices_snapshot))
        .route("/settings", get(settings_get).patch(settings_patch))
        .route("/strategy/perf", get(strategy_perf))
        .route("/strategy/stats", get(strategy_stats))
        .route("/forecast/btc-15m", get(btc_forecast_15m))
        .route("/logs/execution", get(execution_logs))
        .route("/portfolio/summary", get(portfolio_summary))
        .route("/runs", post(start_run))
        .route("/static/styles.css", get(dashboard_styles))
        .route("/static/app.js", get(dashboard_script))
        .route("/ws/events", get(ws::events_socket))
        .with_state(state)
}

async fn dashboard_index() -> Html<&'static str> {
    Html(ui::index_html())
}

async fn dashboard_styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        ui::styles_css(),
    )
}

async fn dashboard_script() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        ui::app_js(),
    )
}

async fn feed_health(State(state): State<AppState>) -> Json<FeedHealthResponse> {
    Json(state.feed_health())
}

async fn markets_discovered(State(state): State<AppState>) -> Json<DiscoveredMarketsResponse> {
    Json(state.discovered_markets())
}

async fn portfolio_summary(State(state): State<AppState>) -> Json<PortfolioSummary> {
    Json(state.portfolio_summary())
}

async fn prices_snapshot(State(state): State<AppState>) -> Json<PriceSnapshot> {
    Json(state.price_snapshot())
}

async fn strategy_perf(State(state): State<AppState>) -> Json<StrategyPerfSummary> {
    Json(state.strategy_perf_summary())
}

async fn strategy_stats(State(state): State<AppState>) -> Json<StrategyStatsSummary> {
    Json(state.strategy_stats_summary())
}

async fn btc_forecast_15m(State(state): State<AppState>) -> Json<BtcForecastSummary> {
    Json(state.btc_forecast_summary())
}

async fn settings_get(State(state): State<AppState>) -> Json<RuntimeSettings> {
    Json(state.runtime_settings())
}

async fn settings_patch(
    State(state): State<AppState>,
    Json(patch): Json<RuntimeSettingsPatch>,
) -> Result<Json<RuntimeSettings>, (StatusCode, Json<serde_json::Value>)> {
    validate_settings_patch(&state, &patch).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": message.to_string() })),
        )
    })?;

    let settings = state.patch_runtime_settings(patch);
    let log = ExecutionLogEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
        event: "settings_update".to_string(),
        headline: "Settings Updated".to_string(),
        detail: format!(
            "mode={} paused={} lag={} risk={} daily_cap={}",
            match settings.execution_mode {
                crate::state::ExecutionMode::Paper => "paper",
                crate::state::ExecutionMode::Live => "live",
            },
            settings.trading_paused,
            settings.lag_threshold_pct,
            settings.risk_per_trade_pct,
            settings.daily_loss_cap_pct,
        ),
    };
    state.push_execution_log(log.clone(), 500);
    let _ = state.publish_event(RuntimeEvent::execution_log(log));
    let _ = state.publish_event(RuntimeEvent::settings_updated(settings.clone()));
    Ok(Json(settings))
}

fn validate_settings_patch(
    state: &AppState,
    patch: &RuntimeSettingsPatch,
) -> Result<(), &'static str> {
    if let Some(value) = patch.lag_threshold_pct {
        if !value.is_finite() || value <= 0.0 || value > 100.0 {
            return Err("lag_threshold_pct must be > 0 and <= 100");
        }
    }

    if let Some(value) = patch.risk_per_trade_pct {
        if !value.is_finite() || value <= 0.0 || value > 100.0 {
            return Err("risk_per_trade_pct must be > 0 and <= 100");
        }
    }

    if let Some(value) = patch.daily_loss_cap_pct {
        if !value.is_finite() || value <= 0.0 || value > 100.0 {
            return Err("daily_loss_cap_pct must be > 0 and <= 100");
        }
    }

    if let Some(crate::state::ExecutionMode::Live) = patch.execution_mode {
        let settings = state.runtime_settings();
        if !settings.live_feature_enabled {
            return Err("execution_mode=live requires live_feature_enabled=true");
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct ExecutionLogsResponse {
    logs: Vec<ExecutionLogEntry>,
}

async fn execution_logs(State(state): State<AppState>) -> Json<ExecutionLogsResponse> {
    Json(ExecutionLogsResponse {
        logs: state.execution_logs(),
    })
}

#[derive(Debug, Serialize)]
struct StartRunResponse {
    run_id: u64,
}

async fn start_run(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let run_id = state
        .start_run()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = state.publish_event(RuntimeEvent::run_started(run_id));
    let location = format!("/runs/{run_id}");

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(StartRunResponse { run_id }),
    ))
}
