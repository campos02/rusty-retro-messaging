use super::broadcasted_command::BroadcastedCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Ubx;

impl BroadcastedCommand for Ubx {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let _ = command;

        let email = &user.email;
        let personal_message = &user
            .personal_message
            .as_ref()
            .expect("Could not get authenticated user");
        let length = personal_message.len();

        format!("UBX {email} {length}\r\n{personal_message}")
    }
}
