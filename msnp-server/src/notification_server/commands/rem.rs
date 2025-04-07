use super::command::Command;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::contacts::{in_allow_list, in_block_list, in_forward_list};
use crate::schema::groups::dsl::groups;
use crate::schema::users::dsl::users;
use crate::schema::users::guid;
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

pub struct Rem {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Rem {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Rem { pool }
    }

    pub fn get_contact_email(&self, contact_guid: &str) -> String {
        let connection = &mut self.pool.get().unwrap();
        users
            .filter(guid.eq(&contact_guid))
            .select(email)
            .get_result(connection)
            .unwrap()
    }
}

impl Command for Rem {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
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
            "RL" => return Err("Removing from RL".to_string()),
            _ => return Err(format!("201 {tr_id}\r\n")),
        }

        if forward_list {
            // Remove from group
            if args.len() > 4 {
                let contact_guid = contact_email;
                let group_guid = args[4];

                let user_database = users
                    .filter(email.eq(&user.email))
                    .select(User::as_select())
                    .get_result(connection)
                    .unwrap();

                let Ok(group) = groups
                    .filter(crate::schema::groups::guid.eq(&group_guid))
                    .select(Group::as_select())
                    .get_result(connection)
                else {
                    return Err(format!("224 {tr_id}\r\n"));
                };

                let Ok(contact) = Contact::belonging_to(&user_database)
                    .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                    .filter(crate::schema::users::guid.eq(&contact_guid))
                    .select(Contact::as_select())
                    .get_result(connection)
                else {
                    return Err(format!("216 {tr_id}\r\n"));
                };

                if let Ok(member) = GroupMember::belonging_to(&group)
                    .filter(crate::schema::group_members::contact_id.eq(&contact.id))
                    .select(GroupMember::as_select())
                    .get_result(connection)
                {
                    delete(&member).execute(connection).unwrap();
                } else {
                    return Err(format!("225 {tr_id}\r\n"));
                }

                return Ok(vec![format!(
                    "REM {tr_id} {list} {contact_guid} {group_guid}\r\n"
                )]);
            } else {
                let contact_guid = contact_email;

                let user_database = users
                    .filter(email.eq(&user.email))
                    .select(User::as_select())
                    .get_result(connection)
                    .unwrap();

                if let Ok(contact) = Contact::belonging_to(&user_database)
                    .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                    .filter(crate::schema::users::guid.eq(&contact_guid))
                    .select(Contact::as_select())
                    .get_result(connection)
                {
                    let forward_list = false;
                    if !contact.in_forward_list {
                        return Err(format!("216 {tr_id}\r\n"));
                    }

                    diesel::update(&contact)
                        .set(in_forward_list.eq(&forward_list))
                        .execute(connection)
                        .unwrap();

                    let contact_email: String = users
                        .filter(id.eq(&contact.contact_id))
                        .select(email)
                        .get_result(connection)
                        .unwrap();

                    if let Some(contact) = user.contacts.get_mut(&contact_email) {
                        contact.in_forward_list = forward_list;
                    };
                } else {
                    return Err(format!("216 {tr_id}\r\n"));
                };

                return Ok(vec![format!("REM {tr_id} {list} {contact_guid}\r\n")]);
            }
        } else {
            let user_database = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
                .unwrap();

            if let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(crate::schema::contacts::contact_id)))
                .filter(email.eq(&contact_email))
                .select(Contact::as_select())
                .get_result(connection)
            {
                if allow_list {
                    let allow_list = false;
                    if !contact.in_allow_list {
                        return Err(format!("216 {tr_id}\r\n"));
                    }

                    diesel::update(&contact)
                        .set(in_allow_list.eq(&allow_list))
                        .execute(connection)
                        .unwrap();

                    if let Some(contact) = user.contacts.get_mut(contact_email) {
                        contact.in_allow_list = allow_list;
                    };
                }

                if block_list {
                    let block_list = false;
                    if !contact.in_block_list {
                        return Err(format!("216 {tr_id}\r\n"));
                    }

                    diesel::update(&contact)
                        .set(in_block_list.eq(&block_list))
                        .execute(connection)
                        .unwrap();

                    if let Some(contact) = user.contacts.get_mut(contact_email) {
                        contact.in_block_list = block_list;
                    };
                }
            } else {
                return Err(format!("216 {tr_id}\r\n"));
            };

            return Ok(vec![format!("REM {tr_id} {list} {contact_email}\r\n")]);
        }
    }
}
