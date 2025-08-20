use super::process_command::{process_authentication_command, process_command};
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{cvr::Cvr, usr_i::UsrI, usr_s::UsrS},
};
use log::{error, trace, warn};
use sqlx::{MySql, Pool};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_authentication_command(
    protocol_version: usize,
    pool: &Pool<MySql>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<
    (
        Option<AuthenticatedUser>,
        Option<broadcast::Receiver<Message>>,
    ),
    ErrorCommand,
> {
    let command = str::from_utf8(&command).or(Err(ErrorCommand::Disconnect(
        "Command contained invalid UTF-8".to_string(),
    )))?;

    let args: Vec<&str> = command.trim().split(' ').collect();
    trace!("C: {command}");

    match *args.first().unwrap_or(&"") {
        "CVR" => {
            process_command(protocol_version, wr, &Cvr, command).await?;
        }

        "USR" => match *args.get(3).unwrap_or(&"") {
            "I" => {
                let usr = UsrI::new(pool.clone());
                process_command(protocol_version, wr, &usr, command).await?;
            }

            "S" => {
                if args.len() < 4 {
                    error!("Command doesn't have enough arguments: {command}");
                    return Err(ErrorCommand::Command("Not enough arguments".to_string()));
                }

                trace!(
                    "C: {} {} {} {} t=xxxxx\r\n",
                    args[0], args[1], args[2], args[3]
                );

                let usr = UsrS::new(pool.clone());
                let (authenticated_user, contact_rx) = process_authentication_command(
                    protocol_version,
                    wr,
                    broadcast_tx,
                    &usr,
                    command,
                )
                .await?;

                return Ok((Some(authenticated_user), Some(contact_rx)));
            }

            _ => {
                let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
                let err = format!("911 {tr_id}\r\n");

                wr.write_all(err.as_bytes())
                    .await
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to client over socket".to_string(),
                    )))?;

                warn!("S: {err}");
                return Err(ErrorCommand::Disconnect(err));
            }
        },

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Ok((None, None))
}
