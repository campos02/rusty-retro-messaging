use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::{
        commands::{iln::Iln, nln::Nln, traits::thread_command::ThreadCommand, ubx::Ubx},
        notification_server::NotificationServer,
    },
};
use log::{trace, warn};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_thread_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    sender: String,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: String,
) -> Result<(), ErrorCommand> {
    let args: Vec<&str> = command.trim().split(' ').collect();

    match args[0] {
        "ILN" => {
            let presence = args[2];
            let contact = args[3];

            if let Some(contact) = authenticated_user.contacts.get_mut(contact) {
                contact.presence = Some(presence.to_string());
            }

            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "NLN" => {
            if command.len() < 2 {
                return Ok(());
            }

            let presence = args[1];
            let contact = args[2];

            if let Some(contact) = authenticated_user.contacts.get_mut(contact) {
                contact.presence = Some(presence.to_string());
            }

            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "FLN" => {
            let contact = args[1].trim();
            if let Some(contact) = authenticated_user.contacts.get_mut(contact) {
                contact.presence = None;
            }

            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "UBX" => {
            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "CHG" => {
            // A user has logged in
            if NotificationServer::verify_contact(&authenticated_user, &sender).is_err() {
                return Ok(());
            }

            let iln_command = Iln::convert(&authenticated_user, &command);
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: iln_command,
            };

            broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");

            let ubx_command = Ubx::convert(&authenticated_user, &command);
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender,
                message: ubx_command,
            };

            broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");
        }

        "ADC" => {
            if NotificationServer::verify_contact(&authenticated_user, &sender).is_err() {
                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {command}");
                return Ok(());
            }

            let nln_command = Nln::convert(&authenticated_user, &command);
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: nln_command,
            };

            broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");

            let ubx_command = Ubx::convert(&authenticated_user, &command);
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender,
                message: ubx_command,
            };

            broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");

            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "REM" => {
            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
        }

        "RNG" => {
            if NotificationServer::verify_contact(&authenticated_user, &sender).is_ok() {
                wr.write_all(command.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                trace!(
                    "S: {} {} {} {} xxxxx {} {}\r\n",
                    args[0], args[1], args[2], args[3], args[5], args[6]
                );
            }
        }

        "OUT" => {
            wr.write_all(command.as_bytes())
                .await
                .expect("Could not send to client over socket");

            trace!("S: {command}");
            return Err(ErrorCommand::Disconnect(
                "User logged in in another computer".to_string(),
            ));
        }

        "GetUserDetails" => {
            if NotificationServer::verify_contact(&authenticated_user, &sender).is_ok() {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: Some(authenticated_user.clone()),
                    protocol_version: Some(protocol_version),
                };

                broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");
            } else {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: None,
                    protocol_version: None,
                };

                broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");
            }
        }

        _ => (),
    };

    Ok(())
}
