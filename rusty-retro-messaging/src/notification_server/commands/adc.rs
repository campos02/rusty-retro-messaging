use super::fln::Fln;
use super::traits::thread_command::ThreadCommand;
use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::message::Message;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::transient::transient_contact::TransientContact;
use crate::schema::contacts::dsl::contacts;
use crate::schema::contacts::{in_allow_list, in_block_list, in_forward_list};
use crate::schema::group_members::dsl::group_members;
use crate::schema::group_members::group_id;
use crate::schema::groups::dsl::groups;
use crate::schema::users::dsl::users;
use crate::{
    models::{contact::Contact, user::User},
    schema::{
        contacts::{display_name, user_id},
        users::{email, id},
    },
};
use diesel::insert_into;
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, MysqlConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};
use tokio::sync::broadcast;

pub struct Adc {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    broadcast_tx: broadcast::Sender<Message>,
}

impl Adc {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
    ) -> Self {
        Adc { pool, broadcast_tx }
    }
}

impl UserCommand for Adc {
    fn handle(
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

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

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
            let contact_email = contact_email.replace("N=", "");

            if contact_email == user.email {
                return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
            }

            let Ok(user_database) = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            };

            let Ok(contact_user) = users
                .filter(email.eq(&contact_email))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("208 {tr_id}\r\n")));
            };

            if let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                .filter(email.eq(&contact_email))
                .select(Contact::as_select())
                .get_result(connection)
            {
                if forward_list {
                    if contact.in_forward_list {
                        return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
                    }

                    if diesel::update(&contact)
                        .set(in_forward_list.eq(&forward_list))
                        .execute(connection)
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

                    if diesel::update(&contact)
                        .set(in_allow_list.eq(&allow_list))
                        .execute(connection)
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

                    if diesel::update(&contact)
                        .set(in_block_list.eq(&block_list))
                        .execute(connection)
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
                    args[4].replace("F=", "")
                } else {
                    contact_email.clone()
                };

                if insert_into(contacts)
                    .values((
                        user_id.eq(&user_database.id),
                        crate::schema::contacts::contact_id.eq(&contact_user.id),
                        display_name.eq(&contact_display_name),
                        in_forward_list.eq(forward_list),
                        in_allow_list.eq(allow_list),
                        in_block_list.eq(block_list),
                    ))
                    .execute(connection)
                    .is_err()
                {
                    return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                }

                user.contacts.insert(
                    contact_email.clone(),
                    TransientContact {
                        email: contact_email.clone(),
                        display_name: contact_display_name.to_string(),
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

            let Ok(user_database) = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            };

            let Ok(group) = groups
                .filter(crate::schema::groups::guid.eq(&group_guid))
                .select(Group::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("224 {tr_id}\r\n")));
            };

            let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                .filter(crate::schema::users::guid.eq(&contact_guid))
                .select(Contact::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("208 {tr_id}\r\n")));
            };

            if GroupMember::belonging_to(&group)
                .filter(crate::schema::group_members::contact_id.eq(&contact.id))
                .select(GroupMember::as_select())
                .get_result(connection)
                .is_ok()
            {
                return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
            } else if insert_into(group_members)
                .values((
                    group_id.eq(&group.id),
                    crate::schema::group_members::contact_id.eq(&contact.id),
                ))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

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
