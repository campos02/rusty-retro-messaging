use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
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
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let group_id = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let new_name = *args
            .get(3)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let database_user =
            sqlx::query!("SELECT id FROM users WHERE email = ? LIMIT 1", *user.email)
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

        if sqlx::query!(
            "SELECT id FROM groups WHERE name = ? AND user_id = ? LIMIT 1",
            new_name,
            database_user.id
        )
        .fetch_one(&self.pool)
        .await
        .is_ok()
        {
            return Err(CommandError::Reply(format!("228 {tr_id}\r\n")));
        }

        if protocol_version >= 10 {
            sqlx::query!(
                "UPDATE groups SET name = ? WHERE guid = ? AND user_id = ?",
                new_name,
                group_id,
                database_user.id
            )
            .execute(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;
        } else {
            sqlx::query!(
                "UPDATE groups SET name = ? WHERE id = ? AND user_id = ?",
                new_name,
                group_id,
                database_user.id
            )
            .execute(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;
        }

        Ok(vec![if protocol_version >= 10 {
            format!("REG {tr_id} {group_id} {new_name}\r\n")
        } else {
            *version_number += 1;
            format!("REG {tr_id} {version_number} {group_id} {new_name}\r\n")
        }])
    }
}
