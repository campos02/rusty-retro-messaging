use crate::models::user::User;
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{SaltString, rand_core},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use log::trace;
use sqlx::{MySql, Pool};

enum HeaderParsingError {
    HeaderNotFound,
    ToStrError,
    ParameterSplitError,
    UrlDecodingError,
}

pub async fn login_server(
    headers: HeaderMap,
    State(pool): State<Pool<MySql>>,
) -> impl IntoResponse {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .ok_or(HeaderParsingError::HeaderNotFound)
        .and_then(|header| header.to_str().or(Err(HeaderParsingError::ToStrError)))
        .or(Err(StatusCode::UNAUTHORIZED))?;

    let passport = authorization
        .split("sign-in=")
        .nth(1)
        .ok_or(HeaderParsingError::ParameterSplitError)
        .and_then(|split| {
            split
                .split(",")
                .next()
                .ok_or(HeaderParsingError::ParameterSplitError)
                .and_then(|passport| {
                    urlencoding::decode(passport).or(Err(HeaderParsingError::UrlDecodingError))
                })
        })
        .or(Err(StatusCode::UNAUTHORIZED))?;

    let pwd = authorization
        .split("pwd=")
        .nth(1)
        .ok_or(HeaderParsingError::ParameterSplitError)
        .and_then(|split| {
            split
                .split(",")
                .next()
                .ok_or(HeaderParsingError::ParameterSplitError)
                .and_then(|password| {
                    urlencoding::decode(password).or(Err(HeaderParsingError::UrlDecodingError))
                })
        })
        .or(Err(StatusCode::UNAUTHORIZED))?;

    let user = sqlx::query_as!(
        User,
        "SELECT id, email, password, display_name, puid, guid, gtc, blp 
        FROM users WHERE email = ? LIMIT 1",
        passport
    )
    .fetch_one(&pool)
    .await
    .or(Err(StatusCode::UNAUTHORIZED))?;

    let parsed_hash =
        PasswordHash::new(&user.password).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    if Argon2::default()
        .verify_password(pwd.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let mut generated_token =
            urlencoding::encode(SaltString::generate(&mut rand_core::OsRng).as_str()).to_string();

        generated_token.insert_str(0, "t=");
        let datetime = (Utc::now() + Duration::hours(24)).naive_utc();

        sqlx::query!(
            "INSERT INTO tokens (token, valid_until, user_id) VALUES (?, ?, ?)",
            generated_token,
            datetime,
            user.id
        )
        .execute(&pool)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

        trace!("Generated token for {}", user.email);
        return Ok([(
            "Authentication-Info",
            format!("Passport1.4 da-status=success,from-PP='{generated_token}'"),
        )]);
    }

    Err(StatusCode::UNAUTHORIZED)
}
