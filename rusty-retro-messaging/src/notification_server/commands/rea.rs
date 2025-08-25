use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::notification_server::commands::traits::user_command::UserCommand;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Rea {
    pool: Pool<MySql>,
}

impl Rea {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

impl UserCommand for Rea {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;

        if protocol_version >= 10 {
            return Err(CommandError::Reply(format!("502 {tr_id}\r\n")));
        }

        let email = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let display_name = args
            .get(3)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let database_user =
            sqlx::query!("SELECT id FROM users WHERE email = ? LIMIT 1", *user.email)
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

        if email != *user.email {
            let contact = sqlx::query!(
                "SELECT contacts.id FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                WHERE guid = ? AND user_id = ?
                LIMIT 1",
                email,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("208 {tr_id}\r\n"))))?;

            if sqlx::query!(
                "UPDATE contacts SET display_name = ? WHERE id = ?",
                *display_name,
                contact.id
            )
            .execute(&self.pool)
            .await
            .is_err()
            {
                return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
            }

            #[allow(clippy::unnecessary_to_owned)]
            if let Some(contact) = user.contacts.get_mut(&email.to_string()) {
                contact.display_name = display_name.clone();
            };
        } else if sqlx::query!(
            "UPDATE users SET display_name = ? WHERE id = ?",
            *display_name,
            database_user.id
        )
        .execute(&self.pool)
        .await
        .is_err()
        {
            return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
        }

        *version_number += 1;
        Ok(vec![format!(
            "REA {tr_id} {version_number} {email} {display_name}\r\n"
        )])
    }
}
