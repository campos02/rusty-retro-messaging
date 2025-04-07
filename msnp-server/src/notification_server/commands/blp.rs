use super::command::Command;
use crate::schema::users::blp;
use crate::schema::users::dsl::users;
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Blp {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Blp {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Blp { pool }
    }
}

impl Command for Blp {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let setting = args[2];

        if setting == "AL" || setting == "BL" {
            diesel::update(users)
                .filter(email.eq(&user.email))
                .set(blp.eq(&setting))
                .execute(connection)
                .unwrap();

            user.blp = setting.to_string();
        }

        Ok(vec![command.to_string()])
    }
}
