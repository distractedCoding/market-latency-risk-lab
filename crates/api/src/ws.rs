use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};

use crate::state::{AppState, RuntimeEvent};

pub async fn events_socket(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| stream_events(socket, state))
}

async fn stream_events(mut socket: WebSocket, state: AppState) {
    let connected = RuntimeEvent::connected();
    if send_event(&mut socket, &connected).await.is_err() {
        return;
    }

    let mut events = state.subscribe_events();
    loop {
        tokio::select! {
            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return,
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if send_event(&mut socket, &event).await.is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        }
    }
}

async fn send_event(socket: &mut WebSocket, event: &RuntimeEvent) -> Result<(), ()> {
    let payload = event_json(event)?;
    socket.send(Message::Text(payload)).await.map_err(|_| ())
}

fn event_json(event: &RuntimeEvent) -> Result<String, ()> {
    serde_json::to_string(event).map_err(|_| ())
}
