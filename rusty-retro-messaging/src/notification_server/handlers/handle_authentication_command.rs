use super::process_command::{process_authentication_command, process_command};
use crate::errors::command_error::CommandError;
use crate::{
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{cvr::Cvr, usr_i::UsrI, usr_s::UsrS},
};
use log::{error, trace, warn};
use sqlx::{MySql, Pool};
use std::error;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_authentication_command(
    protocol_version: usize,
    pool: &Pool<MySql>,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<
    Option<(AuthenticatedUser, broadcast::Receiver<Message>)>,
    Box<dyn error::Error + Send + Sync>,
> {
    let command = str::from_utf8(&command)?;
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
                    return Err(CommandError::NotEnoughArguments.into());
                }

                trace!(
                    "C: {} {} {} {} t=xxxxx\r\n",
                    args[0], args[1], args[2], args[3]
                );

                let usr = UsrS::new(pool.clone());
                return process_authentication_command(
                    protocol_version,
                    wr,
                    broadcast_tx,
                    &usr,
                    command,
                )
                .await;
            }

            _ => {
                let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
                let err = format!("201 {tr_id}\r\n");

                wr.write_all(err.as_bytes()).await?;
                error!("S: {err}");
                return Err(CommandError::ReplyAndDisconnect(err).into());
            }
        },

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Ok(None)
}
