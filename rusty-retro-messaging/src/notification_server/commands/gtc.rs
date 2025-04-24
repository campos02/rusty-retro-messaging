use super::traits::authenticated_command::AuthenticatedCommand;
use crate::error_command::ErrorCommand;
use crate::schema::users::dsl::users;
use crate::schema::users::gtc;
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Gtc {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Gtc {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Gtc { pool }
    }
}

impl AuthenticatedCommand for Gtc {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let setting = args[2];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        if setting == "A" || setting == "N" {
            if diesel::update(users)
                .filter(email.eq(&user.email))
                .set(gtc.eq(&setting))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }
        }

        Ok(vec![command.to_string()])
    }
}
