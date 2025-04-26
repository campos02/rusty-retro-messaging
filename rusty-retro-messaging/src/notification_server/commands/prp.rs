use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::schema::users::dsl::{display_name, users};
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Prp {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Prp {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Prp { pool }
    }
}

impl UserCommand for Prp {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let parameter = args[2];
        let user_display_name = args[3];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        if parameter == "MFN" {
            if diesel::update(users)
                .filter(email.eq(&user.email))
                .set(display_name.eq(&user_display_name))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            user.display_name = user_display_name.to_string();
        }

        Ok(vec![command.to_string()])
    }
}
