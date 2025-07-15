use crate::models::code::Code;
use crate::schema::codes::code;
use crate::schema::codes::dsl::codes;
use crate::schema::users::{blp, display_name, dsl::users, gtc, guid, puid};
use crate::schema::users::{email, password};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    dsl::insert_into,
    r2d2::{ConnectionManager, Pool},
};
use diesel::{QueryDsl, SelectableHelper};
use email_address::EmailAddress;
use log::trace;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CreateUser {
    email: String,
    password: String,
    password_confirmation: String,
    code: String,
}

pub(crate) async fn register(
    State(pool): State<Pool<ConnectionManager<MysqlConnection>>>,
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

    let connection = &mut pool.get().expect("Could not get connection from pool");

    if users
        .filter(email.eq(&payload.email))
        .select(email)
        .get_result::<String>(connection)
        .is_ok()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(String::from("User already registered")),
        );
    }

    if codes
        .filter(code.eq(&payload.code))
        .select(Code::as_select())
        .get_result(connection)
        .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(String::from("Code not found")),
        );
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .expect("Could not hash password")
        .to_string();

    let passport_id = OsRng.next_u64();
    let user_guid = guid_create::GUID::rand().to_string().to_lowercase();

    insert_into(users)
        .values((
            email.eq(&payload.email),
            password.eq(&password_hash),
            display_name.eq(&payload.email),
            puid.eq(&passport_id),
            guid.eq(&user_guid),
            gtc.eq(&"A"),
            blp.eq(&"AL"),
        ))
        .execute(connection)
        .expect("Could not insert new user");

    trace!("{} registered", payload.email);

    (
        StatusCode::OK,
        Json(String::from("User created sucessfully")),
    )
}
