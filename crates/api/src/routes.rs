use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};
use serde::Serialize;

use crate::{state::AppState, ws};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/runs", post(start_run))
        .route("/ws/events", get(ws::events_socket))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct StartRunResponse {
    run_id: u64,
}

async fn start_run(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let run_id = state
        .start_run()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let location = format!("/runs/{run_id}");

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(StartRunResponse { run_id }),
    ))
}
