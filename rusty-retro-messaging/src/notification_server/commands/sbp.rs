use super::traits::authenticated_command::AuthenticatedCommand;
use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::schema::users::dsl::users;
use crate::{
    models::{contact::Contact, user::User},
    schema::{
        contacts::{contact_id, display_name},
        users::{email, guid, id},
    },
};
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, MysqlConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
    r2d2::{ConnectionManager, Pool},
};

pub struct Sbp {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Sbp {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Sbp { pool }
    }
}

impl AuthenticatedCommand for Sbp {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let parameter = args[3];
        let contact_display_name = args[4];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
        };

        if parameter == "MFN" {
            let Ok(user_database) = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            };

            let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(contact_id)))
                .filter(guid.eq(&args[2]))
                .select(Contact::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Command(format!("208 {tr_id}\r\n")));
            };

            let Ok(contact_email) = users
                .filter(id.eq(&contact.contact_id))
                .select(email)
                .get_result::<String>(connection)
            else {
                return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
            };

            if diesel::update(&contact)
                .set(display_name.eq(&contact_display_name))
                .execute(connection)
                .is_err()
            {
                return Err(ErrorCommand::Command(format!("603 {tr_id}\r\n")));
            }

            if let Some(contact) = user.contacts.get_mut(&contact_email) {
                contact.display_name = contact_display_name.to_string();
            };
        }

        Ok(vec![command.to_string()])
    }
}
