use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::codes)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Code {
    pub id: i32,
    pub code: String,
}
