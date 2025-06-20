use super::traits::thread_command::ThreadCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Fln;

impl ThreadCommand for Fln {
    fn convert(user: &AuthenticatedUser, command: &str) -> String {
        let _ = command;
        let email = &user.email;

        format!("FLN {email}\r\n")
    }
}
