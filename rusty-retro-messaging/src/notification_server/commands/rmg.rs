use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use sqlx::{MySql, Pool};

pub struct Rmg {
    pool: Pool<MySql>,
}

impl Rmg {
    pub fn new(pool: Pool<MySql>) -> Self {
        Rmg { pool }
    }
}

impl UserCommand for Rmg {
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

        if group_id == "0" {
            return Err(CommandError::Reply(format!("230 {tr_id}\r\n")));
        }

        if protocol_version >= 10 {
            let group = sqlx::query!(
                "SELECT groups.id, user_id, name, groups.guid FROM groups
                INNER JOIN users ON groups.user_id = users.id
                WHERE groups.guid = ? AND email = ? LIMIT 1",
                group_id,
                *user.email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("224 {tr_id}\r\n"))))?;

            if sqlx::query!("SELECT id FROM group_members WHERE group_id = ?", group.id)
                .fetch_one(&self.pool)
                .await
                .is_ok()
            {
                return Err(CommandError::Reply(format!("226 {tr_id}\r\n")));
            }

            sqlx::query!("DELETE FROM groups WHERE id = ?", group.id)
                .execute(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;
        } else {
            if sqlx::query!("SELECT id FROM group_members WHERE group_id = ?", group_id)
                .fetch_one(&self.pool)
                .await
                .is_ok()
            {
                return Err(CommandError::Reply(format!("226 {tr_id}\r\n")));
            }

            sqlx::query!("DELETE FROM groups WHERE id = ?", group_id)
                .execute(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;
        }

        Ok(vec![if protocol_version >= 10 {
            format!("RMG {tr_id} 1 {group_id}\r\n")
        } else {
            *version_number += 1;
            format!("RMG {tr_id} {version_number} {group_id}\r\n")
        }])
    }
}
