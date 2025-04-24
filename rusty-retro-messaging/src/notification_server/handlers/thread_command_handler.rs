use super::traits::command_handler::CommandHandler;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::{
        commands::{iln::Iln, nln::Nln, traits::broadcasted_command::BroadcastedCommand, ubx::Ubx},
        notification_server::NotificationServer,
    },
};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub struct ThreadCommandHandler {
    broadcast_tx: broadcast::Sender<Message>,
    pub authenticated_user: AuthenticatedUser,
    protocol_version: usize,
}

impl ThreadCommandHandler {
    pub fn new(
        broadcast_tx: broadcast::Sender<Message>,
        authenticated_user: AuthenticatedUser,
        protocol_version: usize,
    ) -> Self {
        ThreadCommandHandler {
            broadcast_tx,
            authenticated_user,
            protocol_version,
        }
    }
}

impl CommandHandler for ThreadCommandHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand> {
        let args: Vec<&str> = command.trim().split(' ').collect();

        match args[0] {
            "ILN" => {
                let presence = args[2];
                let contact = args[3];

                if let Some(contact) = self.authenticated_user.contacts.get_mut(contact) {
                    contact.presence = Some(presence.to_string());
                }

                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "NLN" => {
                if command.len() < 2 {
                    return Ok(());
                }

                let presence = args[1];
                let contact = args[2];

                if let Some(contact) = self.authenticated_user.contacts.get_mut(contact) {
                    contact.presence = Some(presence.to_string());
                }

                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "FLN" => {
                let contact = args[1].trim();
                if let Some(contact) = self.authenticated_user.contacts.get_mut(contact) {
                    contact.presence = None;
                }

                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "UBX" => {
                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "CHG" => {
                // A user has logged in
                if NotificationServer::verify_contact(&self.authenticated_user, &sender).is_err() {
                    return Ok(());
                }

                let iln_command = Iln::convert(&self.authenticated_user, &command);
                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.email.clone(),
                    receiver: sender.clone(),
                    message: iln_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");

                let ubx_command = Ubx::convert(&self.authenticated_user, &command);
                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");
            }

            "ADC" => {
                if NotificationServer::verify_contact(&self.authenticated_user, &sender).is_err() {
                    wr.write_all(command.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    println!("S: {command}");
                    return Ok(());
                }

                let nln_command = Nln::convert(&self.authenticated_user, &command);
                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.email.clone(),
                    receiver: sender.clone(),
                    message: nln_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");

                let ubx_command = Ubx::convert(&self.authenticated_user, &command);
                let thread_message = Message::ToContact {
                    sender: self.authenticated_user.email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");

                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "REM" => {
                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
            }

            "RNG" => {
                if NotificationServer::verify_contact(&self.authenticated_user, &sender).is_ok() {
                    wr.write_all(command.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    println!("S: {command}");
                }
            }

            "OUT" => {
                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {command}");
                return Err(ErrorCommand::Disconnect(
                    "User logged in in another computer".to_string(),
                ));
            }

            "GetUserDetails" => {
                if NotificationServer::verify_contact(&self.authenticated_user, &sender).is_ok() {
                    let thread_message = Message::SendUserDetails {
                        sender: self.authenticated_user.email.clone(),
                        receiver: sender,
                        authenticated_user: Some(self.authenticated_user.clone()),
                        protocol_version: Some(self.protocol_version),
                    };

                    self.broadcast_tx
                        .send(thread_message)
                        .expect("Could not send to broadcast");
                } else {
                    let thread_message = Message::SendUserDetails {
                        sender: self.authenticated_user.email.clone(),
                        receiver: sender,
                        authenticated_user: None,
                        protocol_version: None,
                    };

                    self.broadcast_tx
                        .send(thread_message)
                        .expect("Could not send to broadcast");
                }
            }

            _ => (),
        };

        Ok(())
    }
}
