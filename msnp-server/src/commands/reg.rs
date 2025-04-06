use super::command::Command;
use crate::models::group::Group;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::groups::{guid, name};
use crate::schema::users::dsl::users;
use crate::{models::user::User, schema::users::email};
use diesel::{
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Reg {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Reg {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Reg { pool }
    }
}

impl Command for Reg {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_guid = args[2];
        let new_name = args[3];
        let number = args[4];

        let user_database = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
            .unwrap();

        if let Ok(group) = Group::belonging_to(&user_database)
            .filter(guid.eq(&group_guid))
            .select(Group::as_select())
            .get_result(connection)
        {
            if Group::belonging_to(&user_database)
                .filter(name.eq(&new_name))
                .select(Group::as_select())
                .get_result(connection)
                .is_ok()
            {
                return Err(format!("228 {tr_id}\r\n"));
            }

            diesel::update(&group)
                .set(name.eq(&new_name))
                .execute(connection)
                .unwrap();

            return Ok(vec![format!(
                "REG {tr_id} 1 {group_guid} {new_name} {number}\r\n"
            )]);
        } else {
            return Err(format!("224 {tr_id}\r\n"));
        }
    }
}
