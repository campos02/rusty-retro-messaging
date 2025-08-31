use argon2::{
    Argon2, PasswordHasher,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use email_address::EmailAddress;
use log::trace;
use serde::Deserialize;
use sqlx::{MySql, Pool};
use std::env;

#[derive(Deserialize)]
pub struct CreateUser {
    email: String,
    password: String,
    password_confirmation: String,
    code: String,
}

pub async fn register(
    State(pool): State<Pool<MySql>>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    if payload.password != payload.password_confirmation {
        return (
            StatusCode::BAD_REQUEST,
            Json(String::from("Passwords don't match")),
        );
    }

    if !EmailAddress::is_valid(payload.email.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(String::from("Invalid email address")),
        );
    }

    if sqlx::query!("SELECT id FROM users WHERE email = ?", payload.email)
        .fetch_one(&pool)
        .await
        .is_ok()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(String::from("User already registered")),
        );
    }

    if env::var("USE_REGISTRATION_CODES")
        .map(|var| var.parse::<bool>().unwrap_or(true))
        .unwrap_or(true)
        && sqlx::query!("SELECT id FROM codes WHERE code = ?", payload.code)
            .fetch_one(&pool)
            .await
            .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(String::from("Code not found")),
        );
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let Ok(password_hash) = argon2.hash_password(payload.password.as_bytes(), &salt) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not hash password")),
        );
    };

    let password_hash = password_hash.to_string();
    let passport_id = OsRng.next_u64();
    let user_guid = guid_create::GUID::rand().to_string().to_lowercase();

    if sqlx::query!(
        "INSERT INTO users (email, password, display_name, puid, guid, gtc, blp) VALUES (?, ?, ?, ?, ?, ?, ?)", 
        payload.email,
        password_hash,
        payload.email,
        passport_id,
        user_guid,
        "A",
        "AL"
    )
        .execute(&pool)
        .await
        .is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not register user")),
        );
    }

    trace!("{} registered", payload.email);
    (
        StatusCode::OK,
        Json(String::from("User created successfully")),
    )
}
