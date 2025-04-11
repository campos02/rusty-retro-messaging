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
use regex::Regex;

pub(crate) async fn login_server(
    headers: HeaderMap,
    State(pool): State<Pool<ConnectionManager<MysqlConnection>>>,
) -> impl IntoResponse {
    let connection = &mut pool.get().unwrap();
    let comma_regex = Regex::new("[^,]*").unwrap();

    let authorization = match headers.get(header::AUTHORIZATION) {
        Some(v) => v,
        None => return Err(StatusCode::BAD_REQUEST),
    }
    .to_str()
    .unwrap();

    let split = match authorization.split("sign-in=").nth(1) {
        Some(s) => s,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let passport = urlencoding::decode(comma_regex.find(split).unwrap().as_str()).expect("UTF-8");

    let split = match authorization.split("pwd=").nth(1) {
        Some(s) => s,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let pwd = urlencoding::decode(comma_regex.find(split).unwrap().as_str()).expect("UTF-8");

    let Ok(user) = users
        .filter(email.eq(&passport))
        .select(User::as_select())
        .get_result(connection)
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
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
            .unwrap();

        return Ok([(
            "Authentication-Info",
            format!("Passport1.4 da-status=success,from-PP='{generated_token}'"),
        )]);
    }

    Err(StatusCode::UNAUTHORIZED)
}
