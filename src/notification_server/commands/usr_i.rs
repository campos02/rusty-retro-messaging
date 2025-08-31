use super::traits::command::Command;
use crate::errors::command_error::CommandError;
use sqlx::{MySql, Pool};

pub struct UsrI {
    pool: Pool<MySql>,
}

impl UsrI {
    pub fn new(pool: Pool<MySql>) -> Self {
        UsrI { pool }
    }
}

impl Command for UsrI {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let email = *args
            .get(4)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        if sqlx::query!("SELECT id FROM users WHERE email = ?", email.trim())
            .fetch_one(&self.pool)
            .await
            .is_err()
        {
            return Err(CommandError::ReplyAndDisconnect(format!("911 {tr_id}\r\n")));
        }

        Ok(vec![format!(
            "USR {tr_id} TWN S ct=1,rver=1,wp=FS_40SEC_0_COMPACT,lc=1,id=1\r\n"
        )])
    }
}
