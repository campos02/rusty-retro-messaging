use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use sqlx::{MySql, Pool};

pub struct Gtc {
    pool: Pool<MySql>,
}

impl Gtc {
    pub fn new(pool: Pool<MySql>) -> Self {
        Gtc { pool }
    }
}

impl UserCommand for Gtc {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let setting = *args
            .get(2)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        if (setting == "A" || setting == "N")
            && sqlx::query!(
                "UPDATE users SET gtc = ? WHERE email = ?",
                setting,
                *user.email
            )
            .execute(&self.pool)
            .await
            .is_err()
        {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        }

        Ok(vec![command.to_string()])
    }
}
