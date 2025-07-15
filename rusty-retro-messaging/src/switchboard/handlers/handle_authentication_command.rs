use crate::switchboard::handlers::process_command::process_authentication_command;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{ans::Ans, usr::Usr},
        session::Session,
    },
};
use core::str;
use log::{trace, warn};
use tokio::{
    net::tcp::WriteHalf,
    sync::broadcast::{self},
};

pub async fn handle_authentication_command(
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<(Option<usize>, Option<Session>, Option<AuthenticatedUser>), ErrorCommand> {
    let command_string = unsafe { str::from_utf8_unchecked(&command) };
    let command_string = command_string
        .lines()
        .next()
        .expect("Could not get command from client message")
        .to_string()
        + "\r\n";

    let args: Vec<&str> = command_string.trim().split(' ').collect();
    match args[0] {
        "USR" => {
            let (protocol_version, session, authenticated_user) =
                process_authentication_command(broadcast_tx, wr, &Usr, &command).await?;

            trace!("C: {} {} {} xxxxx\r\n", args[0], args[1], args[2]);
            return Ok((
                Some(protocol_version),
                Some(session),
                Some(authenticated_user),
            ));
        }

        "ANS" => {
            let (protocol_version, session, authenticated_user) =
                process_authentication_command(broadcast_tx, wr, &Ans, &command).await?;

            trace!("C: {} {} {} xxxxx\r\n", args[0], args[1], args[2]);
            return Ok((
                Some(protocol_version),
                Some(session),
                Some(authenticated_user),
            ));
        }

        _ => warn!("Unmatched command before authentication: {command_string}"),
    };

    Ok((None, None, None))
}
