use axum::Json;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_serde::macros::Serialize;
use sqlx::{MySql, Pool};

#[derive(Serialize)]
pub struct UserResponse {
    email: String,
    display_name: String,
}

pub async fn user(headers: HeaderMap, State(pool): State<Pool<MySql>>) -> impl IntoResponse {
    let token = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_str()
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .replace("Bearer ", "");

    let Ok(user) = sqlx::query!(
        "SELECT email, display_name FROM users INNER JOIN tokens ON tokens.user_id = users.id
        WHERE token = ? LIMIT 1",
        token
    )
    .fetch_one(&pool)
    .await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Json(UserResponse {
        email: user.email,
        display_name: user.display_name,
    }))
}
