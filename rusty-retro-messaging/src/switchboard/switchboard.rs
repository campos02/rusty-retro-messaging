use crate::receive_split::receive_split;
use crate::switchboard::commands::bye;
use crate::switchboard::handlers::handle_authentication_command::handle_authentication_command;
use crate::switchboard::handlers::handle_session_command::handle_session_command;
use crate::{
    Message, error_command::ErrorCommand, models::transient::authenticated_user::AuthenticatedUser,
    switchboard::session::Session,
};
use core::str;
use log::{error, trace};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast::{self},
};

pub struct Switchboard {
    pub broadcast_tx: broadcast::Sender<Message>,
    pub session: Option<Session>,
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

    pub async fn listen(&mut self, socket: &mut TcpStream) -> Result<(), ErrorCommand> {
        let (mut rd, mut wr) = socket.split();
        if self.session.is_some() {
            let session_rx = self
                .session_rx
                .as_mut()
                .expect("Could not get session receiver");

            tokio::select! {
                messages = receive_split(&mut rd) => {
                    self.handle_client_commands(&mut wr, messages?).await?
                }

                received = session_rx.recv() => {
                    self.handle_session_message(&mut wr, received.expect("Could not receive from threads")).await?
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
    ) -> Result<(), ErrorCommand> {
        for message in messages {
            if self.session.is_none() {
                let (protocol_version, session, authenticated_user) =
                    handle_authentication_command(&self.broadcast_tx, wr, message).await?;

                self.protocol_version = protocol_version;
                self.authenticated_user = authenticated_user;
                self.session = session;

                if let Some(session) = &self.session {
                    self.session_rx = Some(session.session_tx.subscribe());
                }

                continue;
            }

            handle_session_command(
                self.protocol_version
                    .expect("Could not get protocol version"),
                self.authenticated_user
                    .as_mut()
                    .expect("Could not get authenticated user"),
                self.session.as_mut().expect("Could not get session"),
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
    ) -> Result<(), ErrorCommand> {
        let Message::ToPrincipals { sender, message } = message else {
            return Ok(());
        };

        let messages_string = unsafe { str::from_utf8_unchecked(&message) };
        let command = messages_string
            .lines()
            .next()
            .expect("Could not get command from session message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();
        let Some(principal) = args.get(1) else {
            error!("Command doesn't have enough arguments: {command}");
            return Ok(());
        };

        if *principal
            == *self
                .authenticated_user
                .as_ref()
                .expect("Could not get authenticated user")
                .email
        {
            return Ok(());
        }

        trace!("Thread {sender}: {command}");
        match *args.first().unwrap_or(&"") {
            "MSG" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                trace!("S: {command}");
            }

            "JOI" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                trace!("S: {command}");
            }

            "BYE" => {
                wr.write_all(&message)
                    .await
                    .expect("Could not send to client over socket");

                trace!("S: {command}");
            }
            _ => (),
        };

        Ok(())
    }

    pub async fn send_bye_to_principals(&mut self, idling: bool) {
        let authenticated_user = self
            .authenticated_user
            .as_mut()
            .expect("Could not get authenticated user");
        {
            let mut principals = self
                .session
                .as_ref()
                .expect("Could not get session")
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            principals.remove(&authenticated_user.email);
        }

        let mut bye_command = bye::generate(
            self.protocol_version
                .expect("Could not get protocol version"),
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
            session
                .session_tx
                .send(message)
                .expect("Could not send to session");
        }
    }
}
