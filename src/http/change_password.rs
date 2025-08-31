use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_serde::macros::Deserialize;
use sqlx::{MySql, Pool};

#[derive(Deserialize)]
pub struct ChangePassword {
    current_password: String,
    new_password: String,
}

pub async fn change_password(
    headers: HeaderMap,
    State(pool): State<Pool<MySql>>,
    Json(payload): Json<ChangePassword>,
) -> impl IntoResponse {
    if payload.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(String::from(
                "New password must be at least 8 characters long",
            )),
        ));
    }

    if payload.current_password == payload.new_password {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(String::from(
                "New password can't be the same as the current password",
            )),
        ));
    }

    let token = headers
        .get(AUTHORIZATION)
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not get token")),
        ))?
        .to_str()
        .or(Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not get token")),
        )))?
        .replace("Bearer ", "");

    let Ok(user) = sqlx::query!(
        "SELECT users.id, password FROM users INNER JOIN tokens ON tokens.user_id = users.id
        WHERE token = ? LIMIT 1",
        token
    )
    .fetch_one(&pool)
    .await
    else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("User not found")),
        ));
    };

    let Ok(parsed_hash) = PasswordHash::new(&user.password) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Error hashing password")),
        ));
    };

    let argon2 = Argon2::default();
    argon2
        .verify_password(payload.current_password.as_bytes(), &parsed_hash)
        .or(Err((
            StatusCode::UNAUTHORIZED,
            Json(String::from("Current password incorrect")),
        )))?;

    let salt = SaltString::generate(&mut OsRng);
    let Ok(password_hash) = argon2.hash_password(payload.new_password.as_bytes(), &salt) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not hash password")),
        ));
    };

    sqlx::query!(
        "UPDATE users SET password = ? WHERE id = ?",
        password_hash.to_string(),
        user.id
    )
    .execute(&pool)
    .await
    .or(Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(String::from("Could not change password")),
    )))?;

    // Log out
    sqlx::query!("DELETE FROM tokens WHERE token = ?", token)
        .execute(&pool)
        .await
        .or(Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(String::from("Could not log out")),
        )))?;

    Ok(Json("Password changed successfully"))
}
