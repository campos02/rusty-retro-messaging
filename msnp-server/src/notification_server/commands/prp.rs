use super::command::Command;
use crate::schema::users::dsl::{display_name, users};
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Prp {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Prp {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Prp { pool }
    }
}

impl Command for Prp {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let parameter = args[2];
        let user_display_name = args[3];

        if parameter == "MFN" {
            diesel::update(users)
                .filter(email.eq(&user.email))
                .set(display_name.eq(&user_display_name))
                .execute(connection)
                .unwrap();

            user.display_name = user_display_name.to_string();
        }

        Ok(vec![command.to_string()])
    }
}
