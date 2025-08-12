use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::contact::Contact;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Sbp {
    pool: Pool<MySql>,
}

impl Sbp {
    pub fn new(pool: Pool<MySql>) -> Self {
        Sbp { pool }
    }
}

impl UserCommand for Sbp {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let guid = *args
            .get(2)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        let parameter = *args
            .get(3)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        let contact_display_name = args
            .get(4)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        if parameter == "MFN" {
            let database_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
                *user.email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

            let contact = sqlx::query_as!(
                Contact,
                "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                in_forward_list as `in_forward_list: _`,
                in_allow_list as `in_allow_list: _`,
                in_block_list as `in_block_list: _`
                FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                WHERE guid = ? AND user_id = ?
                LIMIT 1",
                guid,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("208 {tr_id}\r\n"))))?;

            if sqlx::query!(
                "UPDATE contacts SET display_name = ? WHERE id = ?",
                *contact_display_name,
                contact.id
            )
            .execute(&self.pool)
            .await
            .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            if let Some(contact) = user.contacts.get_mut(&contact.email) {
                contact.display_name = contact_display_name;
            };
        }

        Ok(vec![command.to_string()])
    }
}
