use super::{traits::user_command::UserCommand, ubx};
use crate::errors::command_error::CommandError;
use crate::notification_server::verify_contact;
use crate::{message::Message, models::transient::authenticated_user::AuthenticatedUser};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Uux {
    broadcast_tx: broadcast::Sender<Message>,
}

impl Uux {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Uux { broadcast_tx }
    }
}

impl UserCommand for Uux {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let _ = version_number;
        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines
            .next()
            .ok_or(CommandError::NoTrId)?
            .split(' ')
            .collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        if protocol_version < 11 {
            return Err(CommandError::Reply(format!("502 {tr_id}\r\n")));
        }

        let length = args
            .get(2)
            .unwrap_or(&"")
            .parse()
            .or(Err(CommandError::Reply(format!("201 {tr_id}\r\n"))))?;

        let payload = command_lines
            .next()
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let mut payload = payload.to_string();
        payload.truncate(length);
        user.personal_message = Some(Arc::new(payload));

        for email in user.contacts.keys() {
            if verify_contact::verify_contact(user, email).is_err() {
                continue;
            }

            let ubx_command = ubx::convert(user).map_err(CommandError::CouldNotCreateUbx)?;
            let thread_message = Message::ToContact {
                sender: user.email.clone(),
                receiver: email.clone(),
                message: ubx_command,
            };

            self.broadcast_tx
                .send(thread_message)
                .map_err(CommandError::CouldNotSendToBroadcast)?;
        }

        Ok(vec![format!("UUX {tr_id} 0\r\n")])
    }
}
