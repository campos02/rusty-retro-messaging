use super::command::Command;
use crate::models::group::Group;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::groups::dsl::groups;
use crate::schema::groups::{guid, name, user_id};
use crate::schema::users::dsl::users;
use crate::{models::user::User, schema::users::email};
use diesel::insert_into;
use diesel::{
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Adg {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Adg {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Adg { pool }
    }
}

impl Command for Adg {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_name = args[2];

        let user_database = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
            .unwrap();

        if Group::belonging_to(&user_database)
            .filter(name.eq(&group_name))
            .select(Group::as_select())
            .get_result(connection)
            .is_ok()
        {
            return Err(format!("228 {tr_id}\r\n"));
        } else {
            let group_guid = guid_create::GUID::rand().to_string().to_lowercase();

            insert_into(groups)
                .values((
                    user_id.eq(&user_database.id),
                    name.eq(group_name),
                    guid.eq(&group_guid),
                ))
                .execute(connection)
                .unwrap();

            return Ok(vec![format!("ADG {tr_id} 1 {group_name} {group_guid}\r\n")]);
        }
    }
}
