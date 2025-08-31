use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use sqlx::{MySql, Pool};

pub async fn logout(
    headers: HeaderMap,
    State(pool): State<Pool<MySql>>,
) -> Result<StatusCode, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_str()
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .replace("Bearer ", "");

    sqlx::query!("DELETE FROM tokens WHERE token = ?", token)
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(StatusCode::OK)
}
