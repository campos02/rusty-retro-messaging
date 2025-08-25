use crate::errors::command_error::CommandError;
use crate::message::Message;
use crate::models::contact::Contact;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::transient::transient_contact::TransientContact;
use crate::notification_server::commands::fln;
use crate::notification_server::commands::traits::user_command::UserCommand;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Add {
    pool: Pool<MySql>,
    broadcast_tx: broadcast::Sender<Message>,
}

impl Add {
    pub fn new(pool: Pool<MySql>, broadcast_tx: broadcast::Sender<Message>) -> Self {
        Add { pool, broadcast_tx }
    }
}

impl UserCommand for Add {
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

        let list = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let contact_email = *args
            .get(3)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let contact_display_name = *args
            .get(4)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let mut forward_list = false;
        let mut allow_list = false;
        let mut block_list = false;

        match list {
            "FL" => forward_list = true,
            "AL" => allow_list = true,
            "BL" => block_list = true,
            _ => return Err(CommandError::Reply(format!("201 {tr_id}\r\n"))),
        }

        let database_user =
            sqlx::query!("SELECT id FROM users WHERE email = ? LIMIT 1", *user.email)
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

        // Add to group
        if forward_list && args.len() > 5 {
            let group_id = args[5];
            let group = sqlx::query!(
                "SELECT id FROM groups WHERE guid = ? AND user_id = ? LIMIT 1",
                group_id,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("224 {tr_id}\r\n"))))?;

            let contact = sqlx::query!(
                    "SELECT contacts.id FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                    WHERE email = ? AND user_id = ?
                    LIMIT 1",
                    contact_email,
                    database_user.id
                )
                .fetch_one(&self.pool)
                .await
                .or(Err(CommandError::Reply(format!("208 {tr_id}\r\n"))))?;

            if sqlx::query!(
                "SELECT id FROM group_members WHERE group_id = ? AND contact_id = ? LIMIT 1",
                group.id,
                contact.id
            )
            .fetch_one(&self.pool)
            .await
            .is_ok()
            {
                return Err(CommandError::Reply(format!("215 {tr_id}\r\n")));
            }

            sqlx::query!(
                "INSERT INTO group_members (group_id, contact_id) VALUES (?, ?)",
                group.id,
                contact.id
            )
            .execute(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("603 {tr_id}\r\n"))))?;

            *version_number += 1;
            Ok(vec![format!(
                "ADD {tr_id} {list} {version_number} {contact_email} {contact_display_name} {group_id}\r\n"
            )])
        } else {
            let contact_user = sqlx::query!(
                "SELECT id, guid FROM users WHERE email = ? LIMIT 1",
                contact_email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("208 {tr_id}\r\n"))))?;

            let contact_email = Arc::new(contact_email.to_string());
            if let Ok(contact) = sqlx::query_as!(
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
            {
                if forward_list {
                    if contact.in_forward_list {
                        return Err(CommandError::Reply(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_forward_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email.to_string()) {
                        contact.in_forward_list = forward_list;
                    };
                } else if allow_list {
                    if contact.in_allow_list {
                        return Err(CommandError::Reply(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_allow_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email.to_string()) {
                        contact.in_allow_list = allow_list;
                    };
                } else if block_list {
                    if contact.in_block_list {
                        return Err(CommandError::Reply(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_block_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email.to_string()) {
                        contact.in_block_list = block_list;
                    };
                }
            } else {
                let contact_display_name = if forward_list && let Some(display_name) = args.get(4) {
                    Arc::new(display_name.to_string())
                } else {
                    contact_email.clone()
                };

                if sqlx::query!(
                    "INSERT INTO contacts (user_id, contact_id, display_name, in_forward_list, in_allow_list, in_block_list)
                    VALUES (?, ?, ?, ?, ?, ?)",
                    database_user.id,
                    contact_user.id,
                    *contact_display_name,
                    forward_list,
                    allow_list,
                    block_list
                )
                    .execute(&self.pool)
                    .await
                    .is_err()
                {
                    return Err(CommandError::Reply(format!("603 {tr_id}\r\n")));
                }

                user.contacts.insert(
                    contact_email.clone(),
                    TransientContact {
                        email: contact_email.clone(),
                        display_name: contact_display_name,
                        presence: None,
                        msn_object: None,
                        in_forward_list: forward_list,
                        in_allow_list: allow_list,
                        in_block_list: block_list,
                    },
                );
            }

            if forward_list {
                let message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: contact_email.clone(),
                    message: convert(user, version_number),
                };

                self.broadcast_tx
                    .send(message)
                    .map_err(CommandError::CouldNotSendToBroadcast)?;
            } else if block_list {
                let fln_command = fln::convert(user);
                let message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: contact_email.clone(),
                    message: fln_command,
                };

                self.broadcast_tx
                    .send(message)
                    .map_err(CommandError::CouldNotSendToBroadcast)?;
            }

            *version_number += 1;
            Ok(vec![format!(
                "ADD {tr_id} {list} {version_number} {contact_email} {contact_display_name}\r\n"
            )])
        }
    }
}

pub fn convert(user: &AuthenticatedUser, version_number: &mut u32) -> String {
    let user_email = &user.email;
    let user_display_name = &user.display_name;

    *version_number += 1;
    format!("ADD 0 RL {version_number} {user_email} {user_display_name}\r\n")
}
