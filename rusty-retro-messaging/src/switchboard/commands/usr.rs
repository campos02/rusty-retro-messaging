use super::command::Command;
use crate::schema::users::dsl::users;
use crate::{
    models::transient::authenticated_user::AuthenticatedUser,
    schema::users::{display_name, email},
};
use diesel::{
    ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
    r2d2::{ConnectionManager, Pool},
};

pub struct Usr;

impl Command for Usr {
    fn generate(
        &mut self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let connection = &mut pool.get().expect("Could not get connection from pool");
        let user_email = &user.email;
        let user_display_name: String = users
            .filter(email.eq(&user_email))
            .select(display_name)
            .get_result(connection)
            .expect("Could not get authenticated user display name");

        user.display_name = user_display_name.clone();
        format!("USR {tr_id} OK {user_email} {user_display_name}\r\n")
    }
}
