use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json,
    Router,
};
use serde::Serialize;

use crate::{
    state::{AppState, RuntimeEvent},
    ws,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard_index))
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
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        ui::app_js(),
    )
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
