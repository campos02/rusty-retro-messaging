use crate::errors::command_error::CommandError;
use crate::errors::server_error::ServerError;
use crate::receive_split::receive_split;
use crate::switchboard::commands::bye;
use crate::switchboard::handlers::handle_authentication_command::handle_authentication_command;
use crate::switchboard::handlers::handle_session_command::handle_session_command;
use crate::{
    Message, models::transient::authenticated_user::AuthenticatedUser,
    switchboard::session::Session,
};
use core::str;
use log::{error, trace};
use std::error;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast::{self},
};

pub struct Switchboard {
    broadcast_tx: broadcast::Sender<Message>,
    session: Option<Session>,
    session_rx: Option<broadcast::Receiver<Message>>,
    authenticated_user: Option<AuthenticatedUser>,
    protocol_version: Option<usize>,
}

impl Switchboard {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Switchboard {
            broadcast_tx: broadcast_tx.clone(),
            session: None,
            session_rx: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }

    pub async fn listen(
        &mut self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let (mut rd, mut wr) = socket.split();
        if self.session.is_some() {
            let session_rx = self
                .session_rx
                .as_mut()
                .ok_or(ServerError::CouldNotGetSessionReceiver)?;

            tokio::select! {
                messages = receive_split(&mut rd) => {
                    if let Err(error) = self.handle_client_commands(&mut wr, messages?).await {
                        if let Some(session) = self.session.as_ref() {
                            self.broadcast_tx.send(Message::RemoveSession(session.session_id.clone()))?;
                            self.send_bye_to_principals(false).await?;
                        }

                        return Err(error.into());
                    }
                }

                received = session_rx.recv() => {
                    self.handle_session_message(&mut wr, received.map_err(CommandError::CouldNotReceiveFromBroadcast)?).await?
                }
            }
        } else {
            let messages = receive_split(&mut rd).await?;
            self.handle_client_commands(&mut wr, messages).await?;
        }

        Ok(())
    }

    async fn handle_client_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        messages: Vec<Vec<u8>>,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        for message in messages {
            if self.session.is_none() {
                let Some((protocol_version, session, authenticated_user)) =
                    handle_authentication_command(&self.broadcast_tx, wr, message).await?
                else {
                    continue;
                };

                self.protocol_version = Some(protocol_version);
                self.authenticated_user = Some(authenticated_user);
                self.session = Some(session);

                if let Some(session) = &self.session {
                    self.session_rx = Some(session.session_tx.subscribe());
                }

                continue;
            }

            handle_session_command(
                self.protocol_version
                    .ok_or(ServerError::CouldNotGetProtocolVersion)?,
                self.authenticated_user
                    .as_mut()
                    .ok_or(ServerError::CouldNotGetAuthenticatedUser)?,
                self.session
                    .as_mut()
                    .ok_or(ServerError::CouldNotGetSession)?,
                &self.broadcast_tx,
                wr,
                message,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_session_message(
        &mut self,
        wr: &mut WriteHalf<'_>,
        message: Message,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let Message::ToPrincipals { sender, message } = message else {
            return Ok(());
        };

        let messages_string = unsafe { str::from_utf8_unchecked(&message) };
        let command = messages_string
            .lines()
            .next()
            .ok_or(CommandError::CouldNotGetCommand)?
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();
        let Some(principal) = args.get(1) else {
            error!("Could not get principal from command: {command}");
            return Ok(());
        };

        if *principal
            == *self
                .authenticated_user
                .as_ref()
                .ok_or(CommandError::CouldNotGetAuthenticatedUser)?
                .email
        {
            return Ok(());
        }

        trace!("Thread {sender}: {command}");
        match *args.first().unwrap_or(&"") {
            "MSG" => {
                wr.write_all(&message).await?;
                trace!("S: {command}");
            }

            "JOI" => {
                wr.write_all(&message).await?;
                trace!("S: {command}");
            }

            "BYE" => {
                wr.write_all(&message).await?;
                trace!("S: {command}");
            }
            _ => (),
        };

        Ok(())
    }

    pub async fn send_bye_to_principals(
        &mut self,
        idling: bool,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let authenticated_user = self
            .authenticated_user
            .as_mut()
            .ok_or(ServerError::CouldNotGetAuthenticatedUser)?;
        {
            let mut principals = self
                .session
                .as_ref()
                .ok_or(ServerError::CouldNotGetSession)?
                .principals
                .lock()
                .or(Err(ServerError::PrincipalsLockError))?;

            principals.remove(&authenticated_user.email);
        }

        let mut bye_command = bye::generate(
            self.protocol_version
                .ok_or(ServerError::CouldNotGetProtocolVersion)?,
            authenticated_user,
            "",
        );

        if idling {
            bye_command = bye_command.replace("\r\n", " 1\r\n");
        }

        let message = Message::ToPrincipals {
            sender: authenticated_user.email.clone(),
            message: bye_command.as_bytes().to_vec(),
        };

        if let Some(ref session) = self.session {
            session.session_tx.send(message)?;
        }

        Ok(())
    }
}
