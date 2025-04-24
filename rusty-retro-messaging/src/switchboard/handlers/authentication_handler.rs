use super::traits::command_handler::CommandHandler;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{ans::Ans, usr::Usr},
        session::Session,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use tokio::{
    net::tcp::WriteHalf,
    sync::broadcast::{self},
};

pub struct AuthenticationHandler {
    broadcast_tx: broadcast::Sender<Message>,
    pub session: Option<Session>,
    pub authenticated_user: Option<AuthenticatedUser>,
    pub protocol_version: Option<usize>,
}

impl AuthenticationHandler {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        AuthenticationHandler {
            broadcast_tx,
            session: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }
}

impl CommandHandler for AuthenticationHandler {
    async fn handle_command(
        &mut self,
        wr: &mut WriteHalf<'_>,
        base64_command: String,
    ) -> Result<(), ErrorCommand> {
        let bytes = URL_SAFE
            .decode(base64_command.clone())
            .expect("Could not decode client message from base64");

        let command = unsafe { str::from_utf8_unchecked(&bytes) };
        let args: Vec<&str> = command.trim().split(' ').collect();

        println!("C: {command}");
        match args[0] {
            "USR" => {
                let (protocol_version, session, authenticated_user) =
                    Self::run_authentication_command(
                        &self.broadcast_tx,
                        wr,
                        &mut Usr,
                        &base64_command,
                    )
                    .await?;

                self.protocol_version = Some(protocol_version);
                self.session = Some(session);
                self.authenticated_user = Some(authenticated_user);
            }

            "ANS" => {
                let (protocol_version, session, authenticated_user) =
                    Self::run_authentication_command(
                        &self.broadcast_tx,
                        wr,
                        &mut Ans,
                        &base64_command,
                    )
                    .await?;

                self.protocol_version = Some(protocol_version);
                self.session = Some(session);
                self.authenticated_user = Some(authenticated_user);
            }
            _ => println!("Unmatched command before authentication: {command}"),
        };

        Ok(())
    }
}
