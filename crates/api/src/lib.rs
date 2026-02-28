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
        state::{AppState, RuntimeEvent},
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
            .oneshot(Request::get("/static/styles.css").body(Body::empty()).unwrap())
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
        assert_eq!(value.get("run_id").cloned(), Some(Value::Null));

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
}
