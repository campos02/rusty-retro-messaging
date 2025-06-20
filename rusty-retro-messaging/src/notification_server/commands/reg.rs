use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
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

impl UserCommand for Reg {
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
        let new_name = args[3];
        let number = args[4];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        let Ok(user_database) = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
        else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

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
                return Err(ErrorCommand::Command(format!("228 {tr_id}\r\n")));
            }

            if diesel::update(&group)
                .set(name.eq(&new_name))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            Ok(vec![format!(
                "REG {tr_id} 1 {group_guid} {new_name} {number}\r\n"
            )])
        } else {
            Err(ErrorCommand::Command(format!("224 {tr_id}\r\n")))
        }
    }
}
