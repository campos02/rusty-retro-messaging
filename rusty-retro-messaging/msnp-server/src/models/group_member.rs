use crate::models::{contact::Contact, group::Group};
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(Contact))]
#[diesel(table_name = crate::schema::group_members)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct GroupMember {
    pub id: i32,
    pub group_id: i32,
    pub contact_id: i32,
}
