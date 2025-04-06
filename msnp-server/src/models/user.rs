use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub puid: u64,
    pub guid: String,
    pub gtc: String,
    pub blp: String,
}
