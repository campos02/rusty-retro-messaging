use argon2::password_hash::rand_core;
use argon2::password_hash::rand_core::RngCore;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_serde::macros::Deserialize;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use serde_json::json;
use sqlx::{MySql, Pool};

#[derive(Deserialize)]
pub struct Login {
    email: String,
    password: String,
}

pub async fn login(
    State(pool): State<Pool<MySql>>,
    Json(payload): Json<Login>,
) -> impl IntoResponse {
    if !EmailAddress::is_valid(payload.email.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(String::from("Invalid email address")),
        ));
    }

    let Ok(user) = sqlx::query!(
        "SELECT id, password FROM users WHERE email = ? LIMIT 1",
        payload.email
    )
    .fetch_one(&pool)
    .await
    else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("User not registered")),
        ));
    };

    let Ok(parsed_hash) = PasswordHash::new(&user.password) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Error hashing password")),
        ));
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let mut bytes = [0u8; 88];
        rand_core::OsRng.fill_bytes(&mut bytes);

        let generated_token = URL_SAFE.encode(bytes);
        let now = Utc::now().naive_utc();
        let datetime = now + Duration::hours(24);

        if sqlx::query!(
            "INSERT INTO tokens (token, valid_until, user_id) VALUES (?, ?, ?)",
            generated_token,
            datetime,
            user.id
        )
        .execute(&pool)
        .await
        .is_err()
        {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(String::from("Error creating new token")),
            ));
        }

        Ok((StatusCode::OK, Json(json!({"token": generated_token}))))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("Email or password incorrect")),
        ))
    }
}
