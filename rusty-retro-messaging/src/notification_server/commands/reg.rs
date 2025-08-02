use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use sqlx::{MySql, Pool};

pub struct Reg {
    pool: Pool<MySql>,
}

impl Reg {
    pub fn new(pool: Pool<MySql>) -> Self {
        Reg { pool }
    }
}

impl UserCommand for Reg {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let group_guid = args[2];
        let new_name = args[3];
        let number = args[4];

        let database_user = sqlx::query_as!(
            User,
            "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
            user.email
        )
        .fetch_one(&self.pool)
        .await
        .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

        if sqlx::query!(
            "SELECT id FROM groups WHERE name = ? AND user_id = ? LIMIT 1",
            new_name,
            database_user.id
        )
        .fetch_one(&self.pool)
        .await
        .is_ok()
        {
            return Err(ErrorCommand::Command(format!("228 {tr_id}\r\n")));
        }

        sqlx::query!(
            "UPDATE groups SET name = ? WHERE guid = ? AND user_id = ?",
            new_name,
            group_guid,
            database_user.id
        )
        .execute(&self.pool)
        .await
        .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

        Ok(vec![format!(
            "REG {tr_id} 1 {group_guid} {new_name} {number}\r\n"
        )])
    }
}
