use crate::message::Message;
use axum::http::StatusCode;
use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use tokio::sync::broadcast::{self, error::RecvError};

pub async fn stats(State(broadcast_tx): State<broadcast::Sender<Message>>) -> impl IntoResponse {
    let mut broadcast_rx = broadcast_tx.subscribe();
    broadcast_tx
        .send(Message::GetUsers)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut user_count;
    loop {
        let message = match broadcast_rx.recv().await {
            Ok(msg) => msg,
            Err(err) => {
                if let RecvError::Lagged(_) = err {
                    continue;
                } else {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
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

    Ok(Json(json!({ "users": user_count })))
}
