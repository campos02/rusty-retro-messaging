use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
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

impl UserCommand for Rmg {
    fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_guid = args[2];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        if group_guid == "0" {
            return Err(ErrorCommand::Command(format!("230 {tr_id}\r\n")));
        }

        let Ok(user_database) = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
        else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

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
                return Err(ErrorCommand::Command(format!("226 {tr_id}\r\n")));
            } else {
                if delete(&group).execute(connection).is_err() {
                    return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                }
            }
            Ok(vec![format!("RMG {tr_id} 1 {group_guid}\r\n")])
        } else {
            Err(ErrorCommand::Command(format!("224 {tr_id}\r\n")))
        }
    }
}
