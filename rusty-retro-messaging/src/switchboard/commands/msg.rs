use super::traits::command::Command;
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use core::str;

pub struct Msg;

impl Command for Msg {
    async fn handle(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        command: &Vec<u8>,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;
        let command_string = unsafe { str::from_utf8_unchecked(&command) };
        let command_string = command_string
            .lines()
            .next()
            .expect("Could not get command from client message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let email = &user.email;
        let display_name = &user.display_name;

        let length = args[3];
        let async_msg = str::into_boxed_bytes(Box::from(format!(
            "MSG {email} {display_name} {length}\r\n"
        )));

        let mut command = command.clone();
        command.splice(..command_string.len(), async_msg);

        let message = Message::ToPrincipals {
            sender: email.clone(),
            message: command,
        };

        session
            .session_tx
            .send(message)
            .expect("Could not send to session");

        if args[2] == "A" || args[2] == "D" {
            let tr_id = args[1];
            return Ok(vec![format!("ACK {tr_id}\r\n")]);
        }

        Ok(vec![])
    }
}
