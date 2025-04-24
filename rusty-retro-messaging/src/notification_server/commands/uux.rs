use super::{
    traits::{
        authenticated_command::AuthenticatedCommand, broadcasted_command::BroadcastedCommand,
    },
    ubx::Ubx,
};
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::notification_server::NotificationServer,
};
use tokio::sync::broadcast;

pub struct Uux {
    broadcast_tx: broadcast::Sender<Message>,
}

impl Uux {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Uux { broadcast_tx }
    }
}

impl AuthenticatedCommand for Uux {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let mut command_lines = command.lines();
        let args: Vec<&str> = command_lines
            .next()
            .expect("Could not get UUX command from message")
            .split(' ')
            .collect();

        let tr_id = args[1];

        let Ok(length) = args[2].parse() else {
            return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
        };

        let Some(payload) = command_lines.next() else {
            return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
        };

        let mut payload = payload.to_string();
        payload.truncate(length);
        user.personal_message = Some(payload);

        for email in user.contacts.keys() {
            if NotificationServer::verify_contact(&user, &email).is_err() {
                continue;
            }

            let ubx_command = Ubx::convert(&user, &command);
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
