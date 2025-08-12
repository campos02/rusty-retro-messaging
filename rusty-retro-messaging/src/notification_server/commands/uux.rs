use super::{traits::user_command::UserCommand, ubx};
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::notification_server::NotificationServer,
};
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
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines
            .next()
            .expect("Could not get UUX command from message")
            .split(' ')
            .collect();

        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let length = args
            .get(2)
            .unwrap_or(&"")
            .parse()
            .or(Err(ErrorCommand::Command(format!("201 {tr_id}\r\n"))))?;

        let payload = command_lines
            .next()
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        let mut payload = payload.to_string();
        payload.truncate(length);
        user.personal_message = Some(Arc::new(payload));

        for email in user.contacts.keys() {
            if NotificationServer::verify_contact(user, email).is_err() {
                continue;
            }

            let ubx_command = ubx::convert(user, command);
            let thread_message = Message::ToContact {
                sender: user.email.clone(),
                receiver: email.clone(),
                message: ubx_command,
            };

            self.broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");
        }

        Ok(vec![format!("UUX {tr_id} 0\r\n")])
    }
}
