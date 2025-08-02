use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Prp {
    pool: Pool<MySql>,
}

impl Prp {
    pub fn new(pool: Pool<MySql>) -> Self {
        Prp { pool }
    }
}

impl UserCommand for Prp {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let parameter = args[2];
        let user_display_name = args[3];

        if parameter == "MFN" {
            if sqlx::query!(
                "UPDATE users SET display_name = ? WHERE email = ?",
                user_display_name,
                *user.email
            )
            .execute(&self.pool)
            .await
            .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            user.display_name = Arc::new(user_display_name.to_string());
        }

        Ok(vec![command.to_string()])
    }
}
