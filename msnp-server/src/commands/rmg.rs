use super::command::Command;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::groups::{guid, name};
use crate::schema::users::dsl::users;
use crate::{models::user::User, schema::users::email};
use diesel::delete;
use diesel::{
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Rmg {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Rmg {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Rmg { pool }
    }
}

impl Command for Rmg {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_guid = args[2];

        if group_guid == "0" {
            return Err(format!("230 {tr_id}\r\n"));
        }

        let user_database = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
            .unwrap();

        if let Ok(group) = Group::belonging_to(&user_database)
            .filter(guid.eq(&group_guid))
            .or_filter(name.eq("New%20Group"))
            .select(Group::as_select())
            .get_result(connection)
        {
            if GroupMember::belonging_to(&group)
                .select(GroupMember::as_select())
                .get_result(connection)
                .is_ok()
            {
                return Err(format!("226 {tr_id}\r\n"));
            } else {
                delete(&group).execute(connection).unwrap();
            }
            return Ok(vec![format!("RMG {tr_id} 1 {group_guid}\r\n")]);
        } else {
            return Err(format!("224 {tr_id}\r\n"));
        }
    }
}
