use super::command::Command;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Usr;

impl Command for Usr {
    fn generate(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = protocol_version;

        let user_email = &user.email;
        let user_display_name = &user.display_name;

        format!("USR {tr_id} OK {user_email} {user_display_name}\r\n")
    }
}
