use super::traits::command::Command;
use crate::error_command::ErrorCommand;
use crate::models::user::User;
use crate::schema::users::dsl::users;
use crate::schema::users::email;
use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, MysqlConnection, RunQueryDsl};
use diesel::{SelectableHelper, query_dsl::methods::SelectDsl};

pub struct UsrI {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl UsrI {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        UsrI { pool }
    }
}

impl Command for UsrI {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Disconnect(format!("500 {tr_id}\r\n")));
        };

        if users
            .filter(email.eq(args[4].trim()))
            .select(User::as_select())
            .get_result(connection)
            .is_err()
        {
            return Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")));
        }

        return Ok(vec![format!(
            "USR {tr_id} TWN S ct=1,rver=1,wp=FS_40SEC_0_COMPACT,lc=1,id=1\r\n"
        )]);
    }
}
