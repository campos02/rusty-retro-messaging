use super::traits::command::Command;
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;

pub struct Msg;

impl Command for Msg {
    async fn handle(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        base64_command: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;
        let bytes = URL_SAFE
            .decode(base64_command.clone())
            .expect("Could not decode client message from base64");

        let command = unsafe { str::from_utf8_unchecked(&bytes) };
        let command = command
            .lines()
            .next()
            .expect("Could not get command from client message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();

        let email = &user.email;
        let display_name = &user.display_name;

        let length = args[3];
        let async_msg = format!("MSG {email} {display_name} {length}\r\n")
            .as_bytes()
            .to_vec();

        let mut bytes = URL_SAFE
            .decode(base64_command)
            .expect("Could not decode client message from base64");

        bytes.splice(..command.as_bytes().len(), async_msg);
        let base64_command = URL_SAFE.encode(bytes);

        let message = Message::ToPrincipals {
            sender: email.clone(),
            message: base64_command,
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
