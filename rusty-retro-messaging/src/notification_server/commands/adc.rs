use super::fln::Fln;
use super::traits::thread_command::ThreadCommand;
use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::message::Message;
use crate::models::contact::Contact;
use crate::models::group::Group;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::transient::transient_contact::TransientContact;
use crate::models::user::User;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Adc {
    pool: Pool<MySql>,
    broadcast_tx: broadcast::Sender<Message>,
}

impl Adc {
    pub fn new(pool: Pool<MySql>, broadcast_tx: broadcast::Sender<Message>) -> Self {
        Adc { pool, broadcast_tx }
    }
}

impl UserCommand for Adc {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let list = args[2];
        let contact_email = args[3];

        let mut forward_list = false;
        let mut allow_list = false;
        let mut block_list = false;

        match list {
            "FL" => forward_list = true,
            "AL" => allow_list = true,
            "BL" => block_list = true,
            _ => return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n"))),
        }

        if contact_email.starts_with("N=") {
            let contact_email = Arc::new(contact_email.replace("N=", ""));
            if *contact_email == *user.email {
                return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
            }

            let database_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
                *user.email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

            let contact_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
                *contact_email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("208 {tr_id}\r\n"))))?;

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
                        return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_forward_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email) {
                        contact.in_forward_list = forward_list;
                    };
                } else if allow_list {
                    if contact.in_allow_list {
                        return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_allow_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email) {
                        contact.in_allow_list = allow_list;
                    };
                } else if block_list {
                    if contact.in_block_list {
                        return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
                    }

                    if sqlx::query!(
                        "UPDATE contacts SET in_block_list = TRUE WHERE id = ?",
                        contact.id
                    )
                    .execute(&self.pool)
                    .await
                    .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(&contact_email) {
                        contact.in_block_list = block_list;
                    };
                }
            } else {
                let contact_display_name = if forward_list {
                    Arc::new(args[4].replace("F=", ""))
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
                    return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
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
            };

            return if forward_list {
                let contact_guid = contact_user.guid;
                let contact_display_name = args[4];

                let message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: contact_email.clone(),
                    message: Adc::convert(user, command),
                };

                self.broadcast_tx
                    .send(message)
                    .expect("Could not send to broadcast");

                Ok(vec![format!(
                    "ADC {tr_id} {list} N={contact_email} {contact_display_name} C={contact_guid}\r\n"
                )])
            } else {
                if block_list {
                    let fln_command = Fln::convert(user, command);
                    let message = Message::ToContact {
                        sender: user.email.clone(),
                        receiver: contact_email.clone(),
                        message: fln_command,
                    };

                    self.broadcast_tx
                        .send(message)
                        .expect("Could not send to broadcast");
                }

                Ok(vec![format!("ADC {tr_id} {list} N={contact_email}\r\n")])
            };
        // Add to group
        } else if contact_email.starts_with("C=") && list == "FL" {
            let contact_guid = contact_email.replace("C=", "");
            let contact_display_name = args[4].replace("F=", "");
            let group_guid = contact_display_name.replace("C=", "");

            let database_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
                *user.email
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

            let group = sqlx::query_as!(
                Group,
                "SELECT id, user_id, name, guid FROM groups WHERE guid = ? AND user_id = ? LIMIT 1",
                group_guid,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("224 {tr_id}\r\n"))))?;

            let contact = sqlx::query_as!(
                Contact,
                "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                in_forward_list as `in_forward_list: _`,
                in_allow_list as `in_allow_list: _`,
                in_block_list as `in_block_list: _`
                FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                WHERE guid = ? AND user_id = ?
                LIMIT 1",
                contact_guid,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("208 {tr_id}\r\n"))))?;

            if sqlx::query!(
                "SELECT id FROM group_members WHERE group_id = ? AND contact_id = ? LIMIT 1",
                group.id,
                contact.id
            )
            .fetch_one(&self.pool)
            .await
            .is_ok()
            {
                return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
            }

            sqlx::query!(
                "INSERT INTO group_members (group_id, contact_id) VALUES (?, ?)",
                group.id,
                contact.id
            )
            .execute(&self.pool)
            .await
            .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

            return Ok(vec![format!(
                "ADC {tr_id} {list} C={contact_guid} {group_guid}\r\n"
            )]);
        }

        Err(ErrorCommand::Command(format!("208 {tr_id}\r\n")))
    }
}

impl ThreadCommand for Adc {
    fn convert(user: &AuthenticatedUser, command: &str) -> String {
        let _ = command;
        let user_email = &user.email;
        let user_display_name = &user.display_name;

        format!("ADC 0 RL N={user_email} F={user_display_name}\r\n")
    }
}
