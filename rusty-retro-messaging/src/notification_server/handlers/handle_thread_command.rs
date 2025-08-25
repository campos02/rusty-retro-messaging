use crate::errors::thread_command_error::ThreadCommandError;
use crate::notification_server::commands::{iln, nln, ubx};
use crate::notification_server::verify_contact;
use crate::{message::Message, models::transient::authenticated_user::AuthenticatedUser};
use log::{trace, warn};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_thread_command(
    protocol_version: u32,
    authenticated_user: &mut AuthenticatedUser,
    version_number: &mut u32,
    sender: Arc<String>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<&str> = command.trim().split(' ').collect();
    match *args.first().unwrap_or(&"") {
        "ILN" => {
            trace!("Thread {sender}: {command}");
            if args.len() < 4 {
                return Err(ThreadCommandError::NotEnoughArguments(command).into());
            }

            let presence = args[2];
            let contact = args[3].to_string();

            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = Some(Arc::new(presence.to_string()));
            }

            if args.len() > 6 && protocol_version < 9 {
                wr.write_all(
                    format!(
                        "{} {} {} {} {} {}\r\n",
                        args[0], args[1], args[2], args[3], args[4], args[5]
                    )
                    .as_bytes(),
                )
                .await?;
                trace!("S: {command}");
            } else {
                wr.write_all(command.as_bytes()).await?;
                trace!("S: {command}");
            }
        }

        "NLN" => {
            trace!("Thread {sender}: {command}");
            if args.len() < 3 {
                return Err(ThreadCommandError::NotEnoughArguments(command).into());
            }

            let presence = args[1];
            let contact = args[2].to_string();

            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = Some(Arc::new(presence.to_string()));
            }

            if args.len() > 5 && protocol_version < 9 {
                wr.write_all(
                    format!(
                        "{} {} {} {} {}\r\n",
                        args[0], args[1], args[2], args[3], args[4]
                    )
                    .as_bytes(),
                )
                .await?;
                trace!("S: {command}");
            } else {
                wr.write_all(command.as_bytes()).await?;
                trace!("S: {command}");
            }
        }

        "FLN" => {
            trace!("Thread {sender}: {command}");
            if args.len() < 2 {
                return Err(ThreadCommandError::NotEnoughArguments(command).into());
            }

            let contact = args[1].trim().to_string();
            if let Some(contact) = authenticated_user.contacts.get_mut(&contact) {
                contact.presence = None;
            }

            wr.write_all(command.as_bytes()).await?;
            trace!("S: {command}");
        }

        "UBX" => {
            trace!("Thread {sender}: {command}");
            if protocol_version >= 11 {
                wr.write_all(command.as_bytes()).await?;
                trace!("S: {command}");
            }
        }

        "CHG" => {
            // A user has logged in
            trace!("Thread {sender}: {command}");
            if verify_contact::verify_contact(authenticated_user, &sender).is_err() {
                return Ok(());
            }

            let Ok(iln_command) = iln::convert(protocol_version, authenticated_user, &command)
            else {
                return Ok(());
            };

            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: iln_command,
            };

            broadcast_tx.send(thread_message)?;

            if protocol_version >= 11 {
                let Ok(ubx_command) = ubx::convert(authenticated_user) else {
                    return Ok(());
                };

                let thread_message = Message::ToContact {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                broadcast_tx.send(thread_message)?;
            }
        }

        "ADC" => {
            trace!("Thread {sender}: {command}");
            if verify_contact::verify_contact(authenticated_user, &sender).is_err() {
                wr.write_all(command.as_bytes()).await?;

                warn!("S: {command}");
                return Ok(());
            }

            let Ok(nln_command) = nln::convert(protocol_version, authenticated_user) else {
                return Ok(());
            };

            let thread_message = Message::ToContact {
                sender: authenticated_user.email.clone(),
                receiver: sender.clone(),
                message: nln_command,
            };

            broadcast_tx.send(thread_message)?;

            if protocol_version >= 11 {
                let Ok(ubx_command) = ubx::convert(authenticated_user) else {
                    return Ok(());
                };

                let thread_message = Message::ToContact {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    message: ubx_command,
                };

                broadcast_tx.send(thread_message)?;
            }

            if protocol_version >= 10 || args.len() < 5 {
                wr.write_all(command.as_bytes()).await?;
                trace!("S: {command}");
            } else {
                let mut email = args[3].to_string();
                if email.len() > 2 {
                    email.drain(..2);
                }

                let mut display_name = args[4].to_string();
                if display_name.len() > 2 {
                    display_name.drain(..2);
                }

                *version_number += 1;
                wr.write_all(
                    format!(
                        "{} {} {} {version_number} {email} {display_name}\r\n",
                        "ADD", args[1], args[2]
                    )
                    .as_bytes(),
                )
                .await?;

                trace!("S: {command}");
            }
        }

        "REM" => {
            trace!("Thread {sender}: {command}");
            if protocol_version >= 10 || args.len() < 4 {
                wr.write_all(command.as_bytes()).await?;
                trace!("S: {command}");
            } else {
                *version_number += 1;
                wr.write_all(
                    format!(
                        "{} {} {} {version_number} {}\r\n",
                        args[0], args[1], args[2], args[3]
                    )
                    .as_bytes(),
                )
                .await?;

                trace!("S: {command}");
            }
        }

        "RNG" => {
            if args.len() < 7 {
                return Err(ThreadCommandError::NotEnoughArguments(command).into());
            }

            trace!(
                "Thread {sender}: {} {} {} {} xxxxx {} {}\r\n",
                args[0], args[1], args[2], args[3], args[5], args[6]
            );

            if verify_contact::verify_contact(authenticated_user, &sender).is_ok() {
                wr.write_all(command.as_bytes()).await?;
                trace!(
                    "S: {} {} {} {} xxxxx {} {}\r\n",
                    args[0], args[1], args[2], args[3], args[5], args[6]
                );
            }
        }

        "OUT" => {
            trace!("Thread {sender}: {command}");
            wr.write_all(command.as_bytes()).await?;

            trace!("S: {command}");
            return Err(ThreadCommandError::UserLoggedInOnAnotherComputer.into());
        }

        "GetUserDetails" => {
            trace!("Thread {sender}: {command}");
            if verify_contact::verify_contact(authenticated_user, &sender).is_ok() {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: Some(authenticated_user.clone()),
                    protocol_version: Some(protocol_version),
                };

                broadcast_tx.send(thread_message)?;
            } else {
                let thread_message = Message::SendUserDetails {
                    sender: authenticated_user.email.clone(),
                    receiver: sender,
                    authenticated_user: None,
                    protocol_version: None,
                };

                broadcast_tx.send(thread_message)?;
            }
        }

        _ => (),
    };

    Ok(())
}
