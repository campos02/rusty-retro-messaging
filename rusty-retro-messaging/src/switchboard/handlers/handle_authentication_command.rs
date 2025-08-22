use crate::errors::command_error::CommandError;
use crate::switchboard::handlers::process_command::process_authentication_command;
use crate::{
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{ans::Ans, usr::Usr},
        session::Session,
    },
};
use core::str;
use log::{trace, warn};
use std::error;
use tokio::{
    net::tcp::WriteHalf,
    sync::broadcast::{self},
};

pub async fn handle_authentication_command(
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<Option<(usize, Session, AuthenticatedUser)>, Box<dyn error::Error + Send + Sync>> {
    let command_string = unsafe { str::from_utf8_unchecked(&command) };
    let command_string = command_string
        .lines()
        .next()
        .ok_or(CommandError::CouldNotGetCommand)?
        .to_string()
        + "\r\n";

    let args: Vec<&str> = command_string.trim().split(' ').collect();
    match *args.first().unwrap_or(&"") {
        "USR" => {
            if args.len() < 4 {
                return Err(CommandError::NotEnoughArguments.into());
            }

            trace!("C: {} {} {} xxxxx\r\n", args[0], args[1], args[2]);
            return process_authentication_command(broadcast_tx, wr, &Usr, &command).await;
        }

        "ANS" => {
            if args.len() < 4 {
                return Err(CommandError::NotEnoughArguments.into());
            }

            trace!("C: {} {} {} xxxxx\r\n", args[0], args[1], args[2]);
            return process_authentication_command(broadcast_tx, wr, &Ans, &command).await;
        }

        _ => warn!("Unmatched command before authentication: {command_string}"),
    };

    Ok(None)
}
