use crate::models::user::User;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Group {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub guid: String,
}
