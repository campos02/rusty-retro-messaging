use crate::message::Message;
use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use tokio::sync::broadcast;

pub(crate) async fn stats(
    State(broadcast_tx): State<broadcast::Sender<Message>>,
) -> impl IntoResponse {
    let mut broadcast_rx = broadcast_tx.subscribe();
    broadcast_tx
        .send(Message::GetUsers)
        .expect("Could not send GetUsers Message");

    let mut user_count: u32 = 0;
    while let Ok(message) = broadcast_rx.recv().await {
        if let Message::UserCount(count) = message {
            user_count = count;
            if !broadcast_rx.is_empty() {
                continue;
            }
            break;
        }
    }

    Json(json!({ "users": user_count }))
}
