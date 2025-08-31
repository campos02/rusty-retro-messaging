use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_serde::macros::Deserialize;
use sqlx::{MySql, Pool};

#[derive(Deserialize)]
pub struct DeleteAccount {
    password: String,
}

pub async fn delete_account(
    headers: HeaderMap,
    State(pool): State<Pool<MySql>>,
    Json(payload): Json<DeleteAccount>,
) -> impl IntoResponse {
    let token = headers
        .get(AUTHORIZATION)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_str()
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .replace("Bearer ", "");

    let Ok(user) = sqlx::query!(
        "SELECT users.id, password FROM users INNER JOIN tokens ON tokens.user_id = users.id
        WHERE token = ? LIMIT 1",
        token
    )
    .fetch_one(&pool)
    .await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let parsed_hash =
        PasswordHash::new(&user.password).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .or(Err(StatusCode::UNAUTHORIZED))?;

    sqlx::query!("DELETE FROM tokens WHERE user_id = ?", user.id)
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    sqlx::query!("DELETE FROM contacts WHERE user_id = ?", user.id)
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    sqlx::query!("DELETE FROM groups WHERE user_id = ?", user.id)
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    sqlx::query!("DELETE FROM users WHERE id = ?", user.id)
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json("User deleted successfully"))
}
