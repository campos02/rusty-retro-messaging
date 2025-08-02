use super::traits::command::Command;
use crate::error_command::ErrorCommand;
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
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        if sqlx::query!("SELECT email FROM users WHERE email = ?", args[4].trim())
            .fetch_one(&self.pool)
            .await
            .is_err()
        {
            return Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")));
        }

        Ok(vec![format!(
            "USR {tr_id} TWN S ct=1,rver=1,wp=FS_40SEC_0_COMPACT,lc=1,id=1\r\n"
        )])
    }
}
