use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
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
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let setting = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

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
                return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
            }

            user.blp = Arc::new(setting.to_string());
        }

        Ok(vec![if protocol_version >= 10 {
            command.to_string()
        } else {
            *version_number += 1;
            format!("BLP {tr_id} {version_number} {setting}\r\n")
        }])
    }
}
