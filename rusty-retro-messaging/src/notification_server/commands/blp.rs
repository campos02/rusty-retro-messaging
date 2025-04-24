use super::traits::authenticated_command::AuthenticatedCommand;
use crate::error_command::ErrorCommand;
use crate::schema::users::blp;
use crate::schema::users::dsl::users;
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Blp {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Blp {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Blp { pool }
    }
}

impl AuthenticatedCommand for Blp {
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

        if setting == "AL" || setting == "BL" {
            if diesel::update(users)
                .filter(email.eq(&user.email))
                .set(blp.eq(&setting))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            user.blp = setting.to_string();
        }

        Ok(vec![command.to_string()])
    }
}
