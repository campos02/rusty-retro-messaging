use crate::message::Message;
use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use tokio::sync::broadcast::{self, error::RecvError};

pub async fn stats(State(broadcast_tx): State<broadcast::Sender<Message>>) -> impl IntoResponse {
    let mut broadcast_rx = broadcast_tx.subscribe();
    broadcast_tx
        .send(Message::GetUsers)
        .expect("Could not send GetUsers Message");

    let mut user_count;
    loop {
        let message = match broadcast_rx.recv().await {
            Ok(msg) => msg,
            Err(err) => {
                if let RecvError::Lagged(_) = err {
                    continue;
                } else {
                    panic!("Could not receive from broadcast");
                }
            }
        };

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
