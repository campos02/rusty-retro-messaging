use super::traits::command_handler::CommandHandler;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{cal::Cal, msg::Msg},
        session::Session,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use log::{trace, warn};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::WriteHalf,
    sync::broadcast::{self},
};

pub struct AuthenticatedCommandHandler {
    broadcast_tx: broadcast::Sender<Message>,
    pub session: Session,
    pub authenticated_user: AuthenticatedUser,
    protocol_version: usize,
}

impl AuthenticatedCommandHandler {
    pub fn new(
        broadcast_tx: broadcast::Sender<Message>,
        session: Session,
        authenticated_user: AuthenticatedUser,
        protocol_version: usize,
    ) -> Self {
        AuthenticatedCommandHandler {
            broadcast_tx,
            session,
            authenticated_user,
            protocol_version,
        }
    }
}

impl CommandHandler for AuthenticatedCommandHandler {
    async fn handle_command(
        &mut self,
        wr: &mut WriteHalf<'_>,
        base64_command: String,
    ) -> Result<(), ErrorCommand> {
        let bytes = URL_SAFE
            .decode(base64_command.clone())
            .expect("Could not decode client message from base64");

        let command = unsafe { str::from_utf8_unchecked(&bytes) };
        let command = command
            .lines()
            .next()
            .expect("Could not get command from client message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();

        trace!("C: {command}");
        match args[0] {
            "USR" => {
                let tr_id = args[1];
                let err = format!("911 {tr_id}\r\n");

                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
            }

            "ANS" => {
                let tr_id = args[1];
                let err = format!("911 {tr_id}\r\n");

                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
            }

            "CAL" => {
                let mut cal = Cal::new(self.broadcast_tx.clone());
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    &mut self.session,
                    wr,
                    &mut cal,
                    &base64_command,
                )
                .await?;
            }

            "MSG" => {
                Self::run_authenticated_command(
                    self.protocol_version,
                    &mut self.authenticated_user,
                    &mut self.session,
                    wr,
                    &mut Msg,
                    &base64_command,
                )
                .await?;
            }

            "OUT" => {
                return Err(ErrorCommand::Disconnect("Client disconnected".to_string()));
            }

            _ => warn!("Unmatched command: {command}"),
        };

        Ok(())
    }
}
