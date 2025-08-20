use crate::switchboard::handlers::process_command::process_session_command;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{cal::Cal, msg::Msg},
        session::Session,
    },
};
use core::str;
use log::{trace, warn};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::WriteHalf,
    sync::broadcast::{self},
};

pub async fn handle_session_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    session: &mut Session,
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<(), ErrorCommand> {
    let command_string = unsafe { str::from_utf8_unchecked(&command) };
    let command_string = command_string
        .lines()
        .next()
        .ok_or(ErrorCommand::Command(
            "Could not get command from client message".to_string(),
        ))?
        .to_string()
        + "\r\n";

    let args: Vec<&str> = command_string.trim().split(' ').collect();
    trace!("C: {command_string}");

    match *args.first().unwrap_or(&"") {
        "USR" => {
            let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
            let err = format!("911 {tr_id}\r\n");

            wr.write_all(err.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            warn!("S: {err}");
        }

        "ANS" => {
            let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
            let err = format!("911 {tr_id}\r\n");

            wr.write_all(err.as_bytes())
                .await
                .or(Err(ErrorCommand::Disconnect(
                    "Could not send to client over socket".to_string(),
                )))?;

            warn!("S: {err}");
        }

        "CAL" => {
            let cal = Cal::new(broadcast_tx.clone());
            process_session_command(
                protocol_version,
                authenticated_user,
                session,
                wr,
                &cal,
                &command,
            )
            .await?;
        }

        "MSG" => {
            process_session_command(
                protocol_version,
                authenticated_user,
                session,
                wr,
                &Msg,
                &command,
            )
            .await?;
        }

        "OUT" => {
            return Err(ErrorCommand::Disconnect("Client disconnected".to_string()));
        }

        _ => warn!("Unmatched command: {command_string}"),
    };

    Ok(())
}
