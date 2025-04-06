use crate::models::user::User;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::tokens)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Token {
    pub id: i32,
    pub token: String,
    pub valid_until: NaiveDateTime,
    pub user_id: i32,
}
