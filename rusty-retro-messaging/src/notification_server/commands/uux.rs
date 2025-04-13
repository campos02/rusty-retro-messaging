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
        let args: Vec<&str> = command_lines
            .next()
            .expect("Could not get UUX command from message")
            .split(' ')
            .collect();

        let tr_id = args[1];

        let Ok(length) = args[2].parse() else {
            return Err(format!("201 {tr_id}\r\n"));
        };

        let Some(payload) = command_lines.next() else {
            return Err(format!("201 {tr_id}\r\n"));
        };

        let mut payload = payload.to_string();
        payload.truncate(length);
        user.personal_message = Some(payload);

        Ok(vec![format!("UUX {tr_id} 0\r\n")])
    }
}

impl BroadcastedCommand for Uux {
    fn convert(user: &AuthenticatedUser, command: &String) -> String {
        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines
            .next()
            .expect("Could not get UUX command")
            .split(' ')
            .collect();

        let length = args[2].parse().expect("Invalid UUX length");

        let mut payload = command_lines
            .next()
            .expect("Could not get payload from UUX")
            .to_string();

        payload.truncate(length);

        let email = &user.email;
        format!("UBX {email} {length}\r\n{payload}")
    }
}
