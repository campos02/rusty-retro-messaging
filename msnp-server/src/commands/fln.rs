use super::broadcasted_command::BroadcastedCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Fln;

impl BroadcastedCommand for Fln {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let _ = command;
        let email = &user.email;

        format!("FLN {email}\r\n")
    }
}
