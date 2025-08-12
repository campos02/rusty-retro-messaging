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
        command: &[u8],
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;
        let command_string = unsafe { str::from_utf8_unchecked(command) };
        let command_string = command_string
            .lines()
            .next()
            .expect("Could not get command from client message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let ack_type = *args
            .get(2)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        let length = *args
            .get(3)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        let email = &user.email;
        let display_name = &user.display_name;

        let async_msg = str::into_boxed_bytes(Box::from(format!(
            "MSG {email} {display_name} {length}\r\n"
        )));

        let mut command = command.to_vec();
        command.splice(..command_string.len(), async_msg);

        let message = Message::ToPrincipals {
            sender: email.clone(),
            message: command,
        };

        session
            .session_tx
            .send(message)
            .expect("Could not send to session");

        if ack_type == "A" || ack_type == "D" {
            return Ok(vec![format!("ACK {tr_id}\r\n")]);
        }

        Ok(vec![])
    }
}
