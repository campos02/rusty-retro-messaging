use super::process_command::{process_authentication_command, process_command};
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{cvr::Cvr, usr_i::UsrI, usr_s::UsrS},
};
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use log::{trace, warn};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn handle_authentication_command(
    protocol_version: usize,
    pool: &Pool<ConnectionManager<MysqlConnection>>,
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
    let command = str::from_utf8(&command).expect("Command contained invalid UTF-8");
    let args: Vec<&str> = command.trim().split(' ').collect();

    match args[0] {
        "CVR" => {
            trace!("C: {command}");
            process_command(protocol_version, wr, &Cvr, command).await?;
        }

        "USR" => match args[3] {
            "I" => {
                trace!("C: {command}");
                let usr = UsrI::new(pool.clone());
                process_command(protocol_version, wr, &usr, command).await?;
            }

            "S" => {
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
                trace!("C: {command}");
                let tr_id = args[1];
                let err = format!("911 {tr_id}\r\n");

                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
                return Err(ErrorCommand::Disconnect(err));
            }
        },

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Ok((None, None))
}
