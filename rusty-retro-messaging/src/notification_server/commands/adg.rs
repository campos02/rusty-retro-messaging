use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use sqlx::{MySql, Pool};

pub struct Adg {
    pool: Pool<MySql>,
}

impl Adg {
    pub fn new(pool: Pool<MySql>) -> Self {
        Adg { pool }
    }
}

impl UserCommand for Adg {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let group_name = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("228 {tr_id}\r\n")))?;

        let database_user = sqlx::query_as!(
            User,
            "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
            *user.email
        )
        .fetch_one(&self.pool)
        .await
        .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

        if sqlx::query!("SELECT id FROM groups WHERE name = ?", group_name)
            .fetch_one(&self.pool)
            .await
            .is_ok()
        {
            Err(CommandError::Reply(format!("228 {tr_id}\r\n")))
        } else {
            let group_guid = guid_create::GUID::rand().to_string().to_lowercase();
            sqlx::query!(
                "INSERT INTO groups (user_id, name, guid) VALUES (?, ?, ?)",
                database_user.id,
                group_name,
                group_guid
            )
            .execute(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

            Ok(vec![format!("ADG {tr_id} 1 {group_name} {group_guid}\r\n")])
        }
    }
}
