use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::models::group::Group;
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
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let group_guid = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        if group_guid == "0" {
            return Err(CommandError::Reply(format!("230 {tr_id}\r\n")));
        }

        let group = sqlx::query_as!(
            Group,
            "SELECT groups.id, user_id, name, groups.guid FROM groups
            INNER JOIN users ON groups.user_id = users.id
            WHERE groups.guid = ? AND email = ? LIMIT 1",
            group_guid,
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

        Ok(vec![format!("RMG {tr_id} 1 {group_guid}\r\n")])
    }
}
