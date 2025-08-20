use crate::notification_server::commands::{iln, nln, ubx};
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::notification_server::NotificationServer,
};
use log::{error, trace, warn};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_thread_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    sender: Arc<String>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: String,
) -> Result<(), ErrorCommand> {
    let args: Vec<&str> = command.trim().split(' ').collect();
    match *args.first().unwrap_or(&"") {
        "ILN" => {
            trace!("Thread {sender}: {command}");

            if args.len() < 4 {
                error!("Command doesn't have enough arguments: {command}");
                return Ok(());
            }

            let presence = args[2];
            let contact = args[3].to_string();

            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = Some(Arc::new(presence.to_string()));
            }

            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "NLN" => {
            trace!("Thread {sender}: {command}");

            if args.len() < 3 {
                error!("Command doesn't have enough arguments: {command}");
                return Ok(());
            }

            let presence = args[1];
            let contact = args[2].to_string();

            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = Some(Arc::new(presence.to_string()));
            }

            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "FLN" => {
            trace!("Thread {sender}: {command}");

            if args.len() < 2 {
                error!("Command doesn't have enough arguments: {command}");
                return Ok(());
            }

            let contact = args[1].trim().to_string();
            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = None;
            }

            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "UBX" => {
            trace!("Thread {sender}: {command}");
            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "CHG" => {
            trace!("Thread {sender}: {command}");

            // A user has logged in
            if NotificationServer::verify_contact(authenticated_user, &sender).is_err() {
                return Ok(());
            }

            let Ok(iln_command) = iln::convert(authenticated_user, &command) else {
                return Ok(());
            };

            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: iln_command,
            };

            broadcast_tx
                .send(thread_message)
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to broadcast".to_string(),
                )))?;

            let ubx_command = ubx::convert(authenticated_user)?;
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender,
                message: ubx_command,
            };

            broadcast_tx
                .send(thread_message)
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to broadcast".to_string(),
                )))?;
        }

        "ADC" => {
            trace!("Thread {sender}: {command}");
            if NotificationServer::verify_contact(authenticated_user, &sender).is_err() {
                wr.write_all(command.as_bytes())
                    .await
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to client over socket".to_string(),
                    )))?;

                warn!("S: {command}");
                return Ok(());
            }

            let nln_command = nln::convert(authenticated_user)?;
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: nln_command,
            };

            broadcast_tx
                .send(thread_message)
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to broadcast".to_string(),
                )))?;

            let ubx_command = ubx::convert(authenticated_user)?;
            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender,
                message: ubx_command,
            };

            broadcast_tx
                .send(thread_message)
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to broadcast".to_string(),
                )))?;

            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "REM" => {
            trace!("Thread {sender}: {command}");
            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
        }

        "RNG" => {
            if args.len() < 7 {
                error!("Command doesn't have enough arguments: {command}");
                return Ok(());
            }

            trace!(
                "Thread {sender}: {} {} {} {} xxxxx {} {}\r\n",
                args[0], args[1], args[2], args[3], args[5], args[6]
            );

            if NotificationServer::verify_contact(authenticated_user, &sender).is_ok() {
                wr.write_all(command.as_bytes())
                    .await
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to client over socket".to_string(),
                    )))?;

                trace!(
                    "S: {} {} {} {} xxxxx {} {}\r\n",
                    args[0], args[1], args[2], args[3], args[5], args[6]
                );
            }
        }

        "OUT" => {
            trace!("Thread {sender}: {command}");
            wr.write_all(command.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            trace!("S: {command}");
            return Err(ErrorCommand::Disconnect(
                "User logged in in another computer".to_string(),
            ));
        }

        "GetUserDetails" => {
            trace!("Thread {sender}: {command}");
            if NotificationServer::verify_contact(authenticated_user, &sender).is_ok() {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: Some(authenticated_user.clone()),
                    protocol_version: Some(protocol_version),
                };

                broadcast_tx
                    .send(thread_message)
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to broadcast".to_string(),
                    )))?;
            } else {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: None,
                    protocol_version: None,
                };

                broadcast_tx
                    .send(thread_message)
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to broadcast".to_string(),
                    )))?;
            }
        }

        _ => (),
    };

    Ok(())
}
