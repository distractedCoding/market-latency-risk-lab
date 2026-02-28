pub mod routes;
pub mod state;

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
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::app;

    #[tokio::test]
    async fn post_runs_starts_new_run() {
        let app = app();

        let response = app
            .oneshot(Request::post("/runs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
