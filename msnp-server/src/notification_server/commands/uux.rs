use super::{broadcasted_command::BroadcastedCommand, command::Command};
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Uux;

impl Command for Uux {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines.next().unwrap().split(' ').collect();
        let tr_id = args[1];

        let length: usize = args[2].parse().unwrap();
        let mut payload = command_lines.next().unwrap().to_string();
        payload.truncate(length);
        user.personal_message = Some(payload);

        Ok(vec![format!("UUX {tr_id} 0\r\n")])
    }
}

impl BroadcastedCommand for Uux {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines.next().unwrap().split(' ').collect();
        let length: usize = args[2].parse().unwrap();

        let mut payload = command_lines.next().unwrap().to_string();
        payload.truncate(length);

        let email = &user.email;
        format!("UBX {email} {length}\r\n{payload}")
    }
}
