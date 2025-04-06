use crate::schema::users::{blp, display_name, dsl::users, gtc, guid, puid};
use crate::schema::users::{email, password};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use axum::{Json, extract::State, response::IntoResponse};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    dsl::insert_into,
    r2d2::{ConnectionManager, Pool},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CreateUser {
    email: String,
    password: String,
}

pub(crate) async fn register(
    State(pool): State<Pool<ConnectionManager<MysqlConnection>>>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let connection = &mut pool.get().unwrap();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
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
        .unwrap();

    "User created sucessfully"
}
