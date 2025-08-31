use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_serde::macros::Deserialize;
use email_address::EmailAddress;
use sqlx::{MySql, Pool};

#[derive(Deserialize)]
pub struct ChangeEmail {
    current_email: String,
    new_email: String,
    password: String,
}

pub async fn change_email(
    State(pool): State<Pool<MySql>>,
    Json(payload): Json<ChangeEmail>,
) -> impl IntoResponse {
    if payload.current_email == payload.new_email {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(String::from(
                "New email can't be the same as the current email",
            )),
        ));
    }

    if !EmailAddress::is_valid(payload.new_email.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(String::from("Invalid email address")),
        ));
    }

    let Ok(user) = sqlx::query!(
        "SELECT id, password FROM users WHERE email = ? LIMIT 1",
        payload.current_email
    )
    .fetch_one(&pool)
    .await
    else {
        return Err((StatusCode::NOT_FOUND, Json(String::from("User not found"))));
    };

    let Ok(parsed_hash) = PasswordHash::new(&user.password) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Error hashing password")),
        ));
    };

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .or(Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("Password incorrect")),
        )))?;

    sqlx::query!(
        "UPDATE users SET email = ? WHERE id = ?",
        payload.new_email,
        user.id
    )
    .execute(&pool)
    .await
    .or(Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(String::from("Could not change email")),
    )))?;

    Ok(Json("Email changed successfully"))
}
