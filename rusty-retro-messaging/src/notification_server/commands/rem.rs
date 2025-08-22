use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::message::Message;
use crate::models::contact::Contact;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use crate::notification_server::commands::nln;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Rem {
    pool: Pool<MySql>,
    broadcast_tx: broadcast::Sender<Message>,
}

impl Rem {
    pub fn new(pool: Pool<MySql>, broadcast_tx: broadcast::Sender<Message>) -> Self {
        Rem { pool, broadcast_tx }
    }
}

impl UserCommand for Rem {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let list = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let contact_email = args
            .get(3)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let mut forward_list = false;
        let mut allow_list = false;
        let mut block_list = false;

        match list {
            "FL" => forward_list = true,
            "AL" => allow_list = true,
            "BL" => block_list = true,
            "RL" => return Err(CommandError::Reply(format!("201 {tr_id}\r\n"))),
            _ => return Err(CommandError::Reply(format!("201 {tr_id}\r\n"))),
        }

        if forward_list {
            // Remove from group
            if args.len() > 4 {
                let contact_guid = contact_email;
                let group_guid = args[4];

                let database_user = sqlx::query_as!(
                    User,
                    "SELECT id, email, password, display_name, puid, guid, gtc, blp FROM users WHERE email = ? LIMIT 1",
                    *user.email
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

                let group = sqlx::query_as!(
                    Group,
                    "SELECT id, user_id, name, guid FROM groups WHERE guid = ? AND user_id = ? LIMIT 1",
                    group_guid,
                    database_user.id
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("224 {tr_id}\r\n"))))?;

                let contact = sqlx::query_as!(
                    Contact,
                    "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                    in_forward_list as `in_forward_list: _`,
                    in_allow_list as `in_allow_list: _`,
                    in_block_list as `in_block_list: _`
                    FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                    WHERE guid = ? AND user_id = ?
                    LIMIT 1",
                    *contact_guid,
                    database_user.id
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("208 {tr_id}\r\n"))))?;

                let group_member = sqlx::query_as!(
                    GroupMember,
                    "SELECT id, group_id, contact_id FROM group_members WHERE group_id = ? AND contact_id = ? LIMIT 1",
                    group.id,
                    contact.id
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("225 {tr_id}\r\n"))))?;

                sqlx::query!("DELETE FROM group_members WHERE id = ?", group_member.id)
                    .execute(&self.pool)
                    .await
                    .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

                Ok(vec![format!(
                    "REM {tr_id} {list} {contact_guid} {group_guid}\r\n"
                )])
            } else {
                let contact_guid = contact_email;
                let database_user = sqlx::query_as!(
                    User,
                    "SELECT id, email, password, display_name, puid, guid, gtc, blp FROM users WHERE email = ? LIMIT 1",
                    *user.email
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

                let contact = sqlx::query_as!(
                    Contact,
                    "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                    in_forward_list as `in_forward_list: _`,
                    in_allow_list as `in_allow_list: _`,
                    in_block_list as `in_block_list: _`
                    FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                    WHERE guid = ? AND user_id = ?
                    LIMIT 1",
                    *contact_guid,
                    database_user.id
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("216 {tr_id}\r\n"))))?;

                if !contact.in_forward_list {
                    return Err(CommandError::Reply(format!("216 {tr_id}\r\n")));
                }

                if sqlx::query!(
                    "UPDATE contacts SET in_forward_list = FALSE WHERE id = ?",
                    contact.id
                )
                .execute(&self.pool)
                .await
                .is_err()
                {
                    return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                }

                if let Some(contact) = user.contacts.get_mut(&contact.email) {
                    contact.in_forward_list = false;
                };

                let reply = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: Arc::new(contact.email),
                    message: convert(user, command),
                };

                self.broadcast_tx
                    .send(reply)
                    .map_err(CommandError::CouldNotSendToBroadcast)?;

                Ok(vec![format!("REM {tr_id} {list} {contact_guid}\r\n")])
            }
        } else {
            let database_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp FROM users WHERE email = ? LIMIT 1",
                *user.email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

            let contact = sqlx::query_as!(
                Contact,
                "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                    in_forward_list as `in_forward_list: _`,
                    in_allow_list as `in_allow_list: _`,
                    in_block_list as `in_block_list: _`
                    FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                    WHERE email = ? AND user_id = ?
                    LIMIT 1",
                *contact_email,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("216 {tr_id}\r\n"))))?;

            if allow_list {
                if !contact.in_allow_list {
                    return Err(CommandError::Reply(format!("216 {tr_id}\r\n")));
                }

                if sqlx::query!(
                    "UPDATE contacts SET in_allow_list = FALSE WHERE id = ?",
                    contact.id
                )
                .execute(&self.pool)
                .await
                .is_err()
                {
                    return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                }

                if let Some(contact) = user.contacts.get_mut(&contact_email) {
                    contact.in_allow_list = false;
                };
            }

            if block_list {
                if !contact.in_block_list {
                    return Err(CommandError::Reply(format!("216 {tr_id}\r\n")));
                }

                if sqlx::query!(
                    "UPDATE contacts SET in_block_list = FALSE WHERE id = ?",
                    contact.id
                )
                .execute(&self.pool)
                .await
                .is_err()
                {
                    return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                }

                if let Some(contact) = user.contacts.get_mut(&contact_email) {
                    contact.in_block_list = false;
                };

                let nln_command = nln::convert(user).map_err(CommandError::CouldNotCreateNln)?;
                let thread_message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: contact_email.clone(),
                    message: nln_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .map_err(CommandError::CouldNotSendToBroadcast)?;
            }

            Ok(vec![format!("REM {tr_id} {list} {contact_email}\r\n")])
        }
    }
}

pub fn convert(user: &AuthenticatedUser, command: &str) -> String {
    let _ = command;
    let user_email = &user.email;

    format!("REM 0 RL N={user_email}\r\n")
}
