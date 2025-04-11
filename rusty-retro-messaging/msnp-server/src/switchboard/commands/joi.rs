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

pub struct Joi;

impl Command for Joi {
    fn generate(
        &mut self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = tr_id;
        let connection = &mut pool.get().unwrap();
        let user_email = &user.email;
        let user_display_name: String = users
            .filter(email.eq(&user_email))
            .select(display_name)
            .get_result(connection)
            .unwrap();

        user.display_name = user_display_name.clone();
        format!("JOI {user_email} {user_display_name}\r\n")
    }
}
