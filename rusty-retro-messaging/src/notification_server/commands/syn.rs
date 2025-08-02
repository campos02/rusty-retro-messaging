use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::models::contact::Contact;
use crate::models::group::Group;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::transient::transient_contact::TransientContact;
use crate::models::user::User;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Syn {
    pool: Pool<MySql>,
}

impl Syn {
    pub fn new(pool: Pool<MySql>) -> Self {
        Syn { pool }
    }
}

impl UserCommand for Syn {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let first_timestamp = args[2];
        let second_timestamp = args[3];

        let database_user = sqlx::query_as!(
            User,
            "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE email = ? LIMIT 1",
            *user.email
        )
        .fetch_one(&self.pool)
        .await
        .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

        let gtc = &database_user.gtc;
        let mut responses = vec![format!("GTC {gtc}\r\n")];

        let blp = Arc::new(database_user.blp);
        user.blp = blp;
        responses.push(format!("BLP {}\r\n", user.blp));

        let display_name = database_user.display_name;
        user.display_name = Arc::new(display_name);
        responses.push(format!("PRP MFN {}\r\n", user.display_name));

        let user_groups = sqlx::query_as!(
            Group,
            "SELECT id, user_id, name, guid FROM groups WHERE user_id = ?",
            database_user.id
        )
        .fetch_all(&self.pool)
        .await
        .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

        let number_of_groups = user_groups.len();
        for group in &user_groups {
            let name = &group.name;
            let guid = &group.guid;
            responses.push(format!("LSG {name} {guid}\r\n"));
        }

        let user_contacts = sqlx::query_as!(
            Contact,
            "SELECT contacts.id, user_id, contact_id, contacts.display_name, email, guid,
                in_forward_list as `in_forward_list: _`,
                in_allow_list as `in_allow_list: _`,
                in_block_list as `in_block_list: _`
                FROM contacts INNER JOIN users ON contacts.contact_id = users.id
                WHERE user_id = ?",
            database_user.id
        )
        .fetch_all(&self.pool)
        .await
        .or(Err(ErrorCommand::Command(format!("603 {tr_id}\r\n"))))?;

        let number_of_contacts = user_contacts.len();
        for contact in user_contacts {
            let mut listbit = 0;
            if contact.in_forward_list {
                listbit += 1;
            }

            if contact.in_allow_list {
                listbit += 2;
            }

            if contact.in_block_list {
                listbit += 4;
            }

            let display_name = Arc::new(contact.display_name);
            let contact_email = Arc::new(contact.email);

            let transient_contact = TransientContact {
                email: contact_email.clone(),
                display_name: display_name.clone(),
                presence: None,
                msn_object: None,
                in_forward_list: contact.in_forward_list,
                in_allow_list: contact.in_allow_list,
                in_block_list: contact.in_block_list,
            };

            user.contacts
                .insert(transient_contact.email.clone(), transient_contact);

            // Make reverse list
            if sqlx::query!(
                "SELECT id FROM contacts WHERE user_id = ? AND contact_id = ? AND in_forward_list = TRUE LIMIT 1",
                contact.id,
                database_user.id
            )
            .fetch_one(&self.pool)
            .await
            .is_ok()
            {
                listbit += 8;
            }

            if !contact.in_forward_list {
                let mut lst = format!("LST N={contact_email} F={display_name} {listbit}\r\n");
                if protocol_version >= 12 {
                    // Only the Windows Live type is supported at the moment
                    lst = lst.replace("\r\n", " 1\r\n");
                }

                responses.push(lst);
                continue;
            }

            let guid = contact.guid;
            let mut group_list = String::from("");

            for group in &user_groups {
                if sqlx::query!(
                    "SELECT id FROM group_members WHERE group_id = ? AND contact_id = ? LIMIT 1",
                    group.id,
                    contact.id
                )
                .fetch_one(&self.pool)
                .await
                .is_ok()
                {
                    let guid = &group.guid;
                    group_list.push_str(format!("{guid},").as_str());
                }
            }

            if let Some(list) = group_list.strip_suffix(",") {
                group_list = list.to_string();
            }

            let mut lst = format!(
                "LST N={contact_email} F={display_name} C={guid} {listbit} {group_list}\r\n"
            );

            if protocol_version >= 12 {
                // Only the Windows Live type is supported at the moment
                lst = format!(
                    "LST N={contact_email} F={display_name} C={guid} {listbit} 1 {group_list}\r\n"
                );
            }

            responses.push(lst);
        }

        responses.insert(0, format!("SYN {tr_id} {first_timestamp} {second_timestamp} {number_of_contacts} {number_of_groups}\r\n"));
        Ok(responses)
    }
}
