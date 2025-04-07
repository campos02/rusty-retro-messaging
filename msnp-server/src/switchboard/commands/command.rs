use crate::models::transient::authenticated_user::AuthenticatedUser;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};

pub trait Command {
    fn generate(
        &mut self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String;
}
