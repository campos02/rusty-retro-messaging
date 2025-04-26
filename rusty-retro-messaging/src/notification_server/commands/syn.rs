use super::traits::user_command::UserCommand;
use crate::error_command::ErrorCommand;
use crate::schema::contacts::dsl::contacts;
use crate::schema::group_members::contact_id as member_contact_id;
use crate::schema::users::dsl::users;
use crate::{models::group_member::GroupMember, schema::contacts::in_forward_list};
use crate::{
    models::transient::{
        authenticated_user::AuthenticatedUser, transient_contact::TransientContact,
    },
    schema::contacts::contact_id,
};
use crate::{
    models::{contact::Contact, group::Group, user::User},
    schema::users::{email, id},
};
use diesel::{
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Syn {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Syn {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Syn { pool }
    }
}

impl UserCommand for Syn {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let first_timestap = args[2];
        let second_timestamp = args[3];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Disconnect(format!("603 {tr_id}\r\n")));
        };

        let Ok(database_user) = users
            .filter(email.eq(&user.email))
            .select(User::as_select())
            .get_result(connection)
        else {
            return Err(ErrorCommand::Disconnect(format!("603 {tr_id}\r\n")));
        };

        let gtc = &database_user.gtc;
        let mut responses = vec![format!("GTC {gtc}\r\n")];

        let blp = &database_user.blp;
        user.blp = blp.clone();
        responses.push(format!("BLP {blp}\r\n"));

        let display_name = &database_user.display_name;
        user.display_name = display_name.clone();
        responses.push(format!("PRP MFN {display_name}\r\n"));

        let Ok(user_groups) = Group::belonging_to(&database_user)
            .select(Group::as_select())
            .load(connection)
        else {
            return Err(ErrorCommand::Disconnect(format!("603 {tr_id}\r\n")));
        };

        let number_of_groups = user_groups.len();
        for group in &user_groups {
            let name = &group.name;
            let guid = &group.guid;
            responses.push(format!("LSG {name} {guid}\r\n"));
        }

        let Ok(user_contacts) = Contact::belonging_to(&database_user)
            .select(Contact::as_select())
            .load(connection)
        else {
            return Err(ErrorCommand::Disconnect(format!("603 {tr_id}\r\n")));
        };

        let number_of_contacts = user_contacts.len();
        for contact in user_contacts {
            let Ok(contact_user) = users
                .filter(id.eq(&contact.contact_id))
                .select(User::as_select())
                .get_result(connection)
            else {
                continue;
            };

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

            let transient_contact = TransientContact {
                email: contact_user.email.clone(),
                display_name: contact.display_name.clone(),
                presence: None,
                msn_object: None,
                in_forward_list: contact.in_forward_list,
                in_allow_list: contact.in_allow_list,
                in_block_list: contact.in_block_list,
            };

            user.contacts
                .insert(transient_contact.email.clone(), transient_contact);

            // Make reverse list
            if Contact::belonging_to(&contact_user)
                .filter(contact_id.eq(&database_user.id))
                .filter(in_forward_list.eq(true))
                .select(Contact::as_select())
                .get_result(connection)
                .is_ok()
            {
                listbit += 8;
            }

            let display_name = contact.display_name;
            let contact_email = contact_user.email;
            if !contact.in_forward_list {
                let mut lst = format!("LST N={contact_email} F={display_name} {listbit}\r\n");

                if protocol_version >= 12 {
                    // Only the Windows Live type is supported at the moment
                    lst = lst.replace("\r\n", " 1\r\n");
                }

                responses.push(lst);
                continue;
            }

            let guid = contact_user.guid;
            let mut group_list = String::from("");

            for group in &user_groups {
                if GroupMember::belonging_to(&group)
                    .inner_join(contacts)
                    .filter(member_contact_id.eq(&contact.id))
                    .select(Contact::as_select())
                    .get_result(connection)
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

        responses.insert(0, format!("SYN {tr_id} {first_timestap} {second_timestamp} {number_of_contacts} {number_of_groups}\r\n"));
        Ok(responses)
    }
}
