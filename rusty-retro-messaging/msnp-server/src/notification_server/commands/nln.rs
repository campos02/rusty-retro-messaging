use super::broadcasted_command::BroadcastedCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Nln;

impl BroadcastedCommand for Nln {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let _ = command;

        let mut msn_object = String::from("");
        if let Some(object) = &user.msn_object {
            let mut object = String::from(object);
            object.insert_str(0, " ");
            msn_object = object;
        }

        let presence = &user.presence.as_ref().unwrap();
        let email = &user.email;
        let display_name = &user.display_name;
        let client_id = &user.client_id.unwrap();
        format!("NLN {presence} {email} {display_name} {client_id}{msn_object}\r\n")
    }
}
