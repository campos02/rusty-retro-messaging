use super::command::Command;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use std::env;

pub struct Rng {
    pub session_id: String,
    pub cki_string: String,
}

impl Command for Rng {
    fn generate(
        &mut self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = tr_id;
        let _ = pool;

        let session_id = &self.session_id;
        let switchboard_ip = env::var("SWITCHBOARD_IP").expect("SWITCHBOARD_IP not set");
        let switchboard_port = env::var("SWITCHBOARD_PORT").expect("SWITCHBOARD_PORT not set");
        let cki_string = &self.cki_string;
        let email = &user.email;
        let display_name = &user.display_name;

        format!(
            "RNG {session_id} {switchboard_ip}:{switchboard_port} CKI {cki_string} {email} {display_name}\r\n"
        )
    }
}
