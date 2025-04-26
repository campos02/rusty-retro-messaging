use super::traits::thread_command::ThreadCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Iln;

impl ThreadCommand for Iln {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let args: Vec<&str> = command.trim().split(' ').collect();

        let mut msn_object = String::from("");
        if let Some(object) = &user.msn_object {
            let mut object = String::from(object);
            object.insert_str(0, " ");
            msn_object = object;
        }

        let tr_id = args[1];
        let presence = &user.presence.as_ref().expect("User has no presence set");
        let email = &user.email;
        let display_name = &user.display_name;
        let client_id = &user.client_id.expect("User has no client id set");
        format!("ILN {tr_id} {presence} {email} {display_name} {client_id}{msn_object}\r\n")
    }
}
