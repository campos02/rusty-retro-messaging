use super::command::Command;
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

impl Command for Sbp {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let parameter = args[3];
        let contact_display_name = args[4];

        if parameter == "MFN" {
            let user_database = users
                .filter(email.eq(&user.email))
                .select(User::as_select())
                .get_result(connection)
                .unwrap();

            let Ok(contact) = Contact::belonging_to(&user_database)
                .inner_join(users.on(id.eq(contact_id)))
                .filter(guid.eq(&args[2]))
                .select(Contact::as_select())
                .get_result(connection)
            else {
                return Err(format!("208 {tr_id}\r\n"));
            };

            let contact_email: String = users
                .filter(id.eq(&contact.contact_id))
                .select(email)
                .get_result(connection)
                .unwrap();

            diesel::update(&contact)
                .set(display_name.eq(&contact_display_name))
                .execute(connection)
                .unwrap();

            if let Some(contact) = user.contacts.get_mut(&contact_email) {
                contact.display_name = contact_display_name.to_string();
            };
        }

        Ok(vec![command.to_string()])
    }
}
