use crate::models::user::User;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::contacts)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Contact {
    pub id: i32,
    pub user_id: i32,
    pub contact_id: i32,
    pub display_name: String,
    pub in_forward_list: bool,
    pub in_allow_list: bool,
    pub in_block_list: bool,
}
