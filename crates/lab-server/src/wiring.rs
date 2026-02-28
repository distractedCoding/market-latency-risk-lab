use axum::{routing::get, Router};

pub fn build_app() -> Router {
    debug_assert!(runtime::module_ready());
    debug_assert!(api::module_ready());
    debug_assert!(ui::module_ready());

    api::app().route("/health", get(healthcheck))
}

async fn healthcheck() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn server_healthcheck_responds_ok() {
        let app = super::build_app();

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn server_preserves_api_routes_from_build_app() {
        let app = super::build_app();

        let response = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
