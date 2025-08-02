use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Blp {
    pool: Pool<MySql>,
}

impl Blp {
    pub fn new(pool: Pool<MySql>) -> Self {
        Blp { pool }
    }
}

impl UserCommand for Blp {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let setting = args[2];

        if setting == "AL" || setting == "BL" {
            if sqlx::query!(
                "UPDATE users SET blp = ? WHERE email = ?",
                setting,
                *user.email
            )
            .execute(&self.pool)
            .await
            .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            user.blp = Arc::new(setting.to_string());
        }

        Ok(vec![command.to_string()])
    }
}
