use super::nln::Nln;
use super::traits::thread_command::ThreadCommand;
use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::message::Message;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::contacts::{in_allow_list, in_block_list, in_forward_list};
use crate::schema::groups::dsl::groups;
use crate::schema::users::dsl::users;
use crate::{
    models::{contact::Contact, user::User},
    schema::users::{email, id},
};
use diesel::delete;
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, MysqlConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};
use tokio::sync::broadcast;

pub struct Rem {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    broadcast_tx: broadcast::Sender<Message>,
}

impl Rem {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
    ) -> Self {
        Rem { pool, broadcast_tx }
    }
}

impl UserCommand for Rem {
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
            "RL" => return Err(ErrorCommand::Disconnect("Removing from RL".to_string())),
            _ => return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n"))),
        }

        if forward_list {
            // Remove from group
            if args.len() > 4 {
                let contact_guid = contact_email;
                let group_guid = args[4];

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
                    return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
                };

                if let Ok(member) = GroupMember::belonging_to(&group)
                    .filter(crate::schema::group_members::contact_id.eq(&contact.id))
                    .select(GroupMember::as_select())
                    .get_result(connection)
                {
                    if delete(&member).execute(connection).is_err() {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }
                } else {
                    return Err(ErrorCommand::Command(format!("225 {tr_id}\r\n")));
                }

                Ok(vec![format!(
                    "REM {tr_id} {list} {contact_guid} {group_guid}\r\n"
                )])
            } else {
                let contact_guid = contact_email;

                let Ok(user_database) = users
                    .filter(email.eq(&user.email))
                    .select(User::as_select())
                    .get_result(connection)
                else {
                    return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                };

                if let Ok(contact) = Contact::belonging_to(&user_database)
                    .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                    .filter(crate::schema::users::guid.eq(&contact_guid))
                    .select(Contact::as_select())
                    .get_result(connection)
                {
                    let forward_list = false;
                    if !contact.in_forward_list {
                        return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
                    }

                    if diesel::update(&contact)
                        .set(in_forward_list.eq(&forward_list))
                        .execute(connection)
                        .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    let Ok(contact_email) = users
                        .filter(id.eq(&contact.contact_id))
                        .select(email)
                        .get_result::<String>(connection)
                    else {
                        return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
                    };

                    if let Some(contact) = user.contacts.get_mut(&contact_email) {
                        contact.in_forward_list = forward_list;
                    };

                    let reply = Message::ToContact {
                        sender: user.email.clone(),
                        receiver: contact_email,
                        message: Rem::convert(&user, &command),
                    };

                    self.broadcast_tx
                        .send(reply)
                        .expect("Could not send to broadcast");
                } else {
                    return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
                };

                Ok(vec![format!("REM {tr_id} {list} {contact_guid}\r\n")])
            }
        } else {
            let Ok(user_database) = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            };

            if let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                .filter(email.eq(&contact_email))
                .select(Contact::as_select())
                .get_result(connection)
            {
                if allow_list {
                    let allow_list = false;
                    if !contact.in_allow_list {
                        return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
                    }

                    if diesel::update(&contact)
                        .set(in_allow_list.eq(&allow_list))
                        .execute(connection)
                        .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(contact_email) {
                        contact.in_allow_list = allow_list;
                    };
                }

                if block_list {
                    let block_list = false;
                    if !contact.in_block_list {
                        return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
                    }

                    if diesel::update(&contact)
                        .set(in_block_list.eq(&block_list))
                        .execute(connection)
                        .is_err()
                    {
                        return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
                    }

                    if let Some(contact) = user.contacts.get_mut(contact_email) {
                        contact.in_block_list = block_list;
                    };

                    let nln_command = Nln::convert(&user, &command);
                    let thread_message = Message::ToContact {
                        sender: user.email.clone(),
                        receiver: contact_email.to_string(),
                        message: nln_command,
                    };

                    self.broadcast_tx
                        .send(thread_message)
                        .expect("Could not send to broadcast");
                }
            } else {
                return Err(ErrorCommand::Command(format!("216 {tr_id}\r\n")));
            };

            Ok(vec![format!("REM {tr_id} {list} {contact_email}\r\n")])
        }
    }
}

impl ThreadCommand for Rem {
    fn convert(user: &AuthenticatedUser, command: &str) -> String {
        let _ = command;
        let user_email = &user.email;

        format!("REM 0 RL N={user_email}\r\n")
    }
}
