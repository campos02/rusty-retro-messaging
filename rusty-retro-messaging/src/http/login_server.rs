use crate::schema::tokens::dsl::tokens;
use crate::schema::users::dsl::users;
use crate::{
    models::user::User,
    schema::{
        tokens::{token, user_id, valid_until},
        users::email,
    },
};
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
use diesel::{
    ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    dsl::insert_into,
    r2d2::{ConnectionManager, Pool},
};
use log::trace;
use regex::Regex;

enum HeaderParsingError {
    HeaderNotFound,
    ToStrError,
    ParameterSplitError,
    CommaRegexError,
    UrlDecodingError,
}

pub(crate) async fn login_server(
    headers: HeaderMap,
    State(pool): State<Pool<ConnectionManager<MysqlConnection>>>,
) -> impl IntoResponse {
    let connection = &mut pool.get().expect("Could not get connection from pool");
    let comma_regex = Regex::new("[^,]*").expect("Could not build regex");

    let Ok(authorization) = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| HeaderParsingError::HeaderNotFound)
        .and_then(|header| {
            header
                .to_str()
                .or_else(|_| Err(HeaderParsingError::ToStrError))
        })
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(passport) = authorization
        .split("sign-in=")
        .nth(1)
        .ok_or_else(|| HeaderParsingError::ParameterSplitError)
        .and_then(|split| {
            comma_regex
                .find(split)
                .ok_or_else(|| HeaderParsingError::CommaRegexError)
                .and_then(|passport| {
                    urlencoding::decode(passport.as_str())
                        .or_else(|_| Err(HeaderParsingError::UrlDecodingError))
                })
        })
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(pwd) = authorization
        .split("pwd=")
        .nth(1)
        .ok_or_else(|| HeaderParsingError::ParameterSplitError)
        .and_then(|split| {
            comma_regex
                .find(split)
                .ok_or_else(|| HeaderParsingError::CommaRegexError)
                .and_then(|passport| {
                    urlencoding::decode(passport.as_str())
                        .or_else(|_| Err(HeaderParsingError::UrlDecodingError))
                })
        })
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(user) = users
        .filter(email.eq(&passport))
        .select(User::as_select())
        .get_result(connection)
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let parsed_hash = PasswordHash::new(&user.password).expect("Could not hash password");
    if Argon2::default()
        .verify_password(pwd.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let mut generated_token =
            urlencoding::encode(SaltString::generate(&mut rand_core::OsRng).as_str()).into_owned();
        generated_token.insert_str(0, "t=");
        let datetime = (Utc::now() + Duration::hours(24)).naive_utc();

        insert_into(tokens)
            .values((
                token.eq(&generated_token),
                valid_until.eq(&datetime),
                user_id.eq(&user.id),
            ))
            .execute(connection)
            .expect("Could not insert token");

        trace!("Generated token for {}", user.email);

        return Ok([(
            "Authentication-Info",
            format!("Passport1.4 da-status=success,from-PP='{generated_token}'"),
        )]);
    }

    Err(StatusCode::UNAUTHORIZED)
}
