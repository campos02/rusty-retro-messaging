use super::traits::thread_command::ThreadCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use std::env;

pub struct Rng<'a> {
    session_id: &'a str,
    cki_string: &'a str,
}

impl<'a> Rng<'a> {
    pub fn new(session_id: &'a str, cki_string: &'a str) -> Self {
        Self {
            session_id,
            cki_string,
        }
    }
}

impl<'a> ThreadCommand for Rng<'a> {
    fn generate(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = protocol_version;
        let _ = tr_id;

        let session_id = &self.session_id;
        let switchboard_ip = env::var("SWITCHBOARD_IP").expect("SWITCHBOARD_IP not set");
        let cki_string = &self.cki_string;
        let email = &user.email;
        let display_name = &user.display_name;

        format!(
            "RNG {session_id} {switchboard_ip}:1864 CKI {cki_string} {email} {display_name}\r\n"
        )
    }
}
