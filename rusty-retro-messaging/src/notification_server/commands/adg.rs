use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
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
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let group_name = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("228 {tr_id}\r\n")))?;

        let database_user =
            sqlx::query!("SELECT id FROM users WHERE email = ? LIMIT 1", *user.email)
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

            Ok(vec![if protocol_version >= 10 {
                format!("ADG {tr_id} 1 {group_name} {group_guid}\r\n")
            } else {
                let group = sqlx::query!("SELECT id FROM groups WHERE guid = ?", group_guid)
                    .fetch_one(&self.pool)
                    .await
                    .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

                *version_number += 1;
                format!(
                    "ADG {tr_id} {version_number} {group_name} {} 0\r\n",
                    group.id
                )
            }])
        }
    }
}
