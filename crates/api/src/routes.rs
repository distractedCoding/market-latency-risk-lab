use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Router,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new().route("/runs", post(start_run)).with_state(state)
}

async fn start_run(State(state): State<AppState>) -> StatusCode {
    let _ = state.start_run();
    StatusCode::CREATED
}
