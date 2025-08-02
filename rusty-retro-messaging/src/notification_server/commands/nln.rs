use super::traits::thread_command::ThreadCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Nln;

impl ThreadCommand for Nln {
    fn convert(user: &AuthenticatedUser, command: &str) -> String {
        let _ = command;

        let presence = &user.presence.as_ref().expect("User has no presence set");
        let email = &user.email;
        let display_name = &user.display_name;
        let client_id = &user.client_id.expect("User has no client id set");

        if let Some(msn_object) = user.msn_object.as_ref() {
            format!("NLN {presence} {email} {display_name} {client_id} {msn_object}\r\n")
        } else {
            format!("NLN {presence} {email} {display_name} {client_id}\r\n")
        }
    }
}
