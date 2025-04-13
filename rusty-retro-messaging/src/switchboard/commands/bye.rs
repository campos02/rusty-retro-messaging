use super::command::Command;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};

pub struct Bye;

impl Command for Bye {
    fn generate(
        &mut self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = tr_id;
        let _ = pool;
        let email = &user.email;

        format!("BYE {email}\r\n")
    }
}
