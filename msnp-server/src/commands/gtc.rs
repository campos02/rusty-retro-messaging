use super::command::Command;
use crate::schema::users::dsl::users;
use crate::schema::users::gtc;
use crate::{models::transient::authenticated_user::AuthenticatedUser, schema::users::email};
use diesel::{
    ExpressionMethods, MysqlConnection, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Gtc {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Gtc {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Gtc { pool }
    }
}

impl Command for Gtc {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let connection = &mut self.pool.get().unwrap();
        let args: Vec<&str> = command.trim().split(' ').collect();
        let setting = args[2];

        if setting == "A" || setting == "N" {
            diesel::update(users)
                .filter(email.eq(&user.email))
                .set(gtc.eq(&setting))
                .execute(connection)
                .unwrap();
        }

        Ok(vec![command.to_string()])
    }
}
