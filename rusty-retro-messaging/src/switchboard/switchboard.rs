use crate::{
    Message,
    error_command::ErrorCommand,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::{bye::Bye, traits::thread_command::ThreadCommand},
        handlers::{
            authentication_handler::AuthenticationHandler,
            session_command_handler::SessionCommandHandler,
            traits::command_handler::CommandHandler,
        },
        session::Session,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use log::trace;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
        let mut buf = vec![0; 1664];

        if self.session.is_some() {
            let session_rx = self
                .session_rx
                .as_mut()
                .expect("Could not get session receiver");

            tokio::select! {
                received = rd.read(&mut buf) => {
                    let Ok(received) = received else {
                        return Err(ErrorCommand::Disconnect("Could not read from client".to_string()));
                    };

                    if received == 0 {
                        return Err(ErrorCommand::Disconnect("Client disconnected".to_string()));
                    }

                    let buf = &buf[..received];
                    self.handle_client_commands(socket, buf.to_vec()).await?
                }

                received = session_rx.recv() => {
                    self.handle_session_message(&mut wr, received.expect("Could not receive from threads")).await?
                }
            }
        } else {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let Ok(received) = received else {
                        return Err(ErrorCommand::Disconnect("Could not read from client".to_string()));
                    };

                    if received == 0 {
                        return Err(ErrorCommand::Disconnect("Client disconnected".to_string()));
                    }

                    let buf = &buf[..received];
                    self.handle_client_commands(socket, buf.to_vec()).await?
                }
            }
        }

        Ok(())
    }

    async fn handle_client_commands(
        &mut self,
        socket: &mut TcpStream,
        mut messages_bytes: Vec<u8>,
    ) -> Result<(), ErrorCommand> {
        let (mut rd, mut wr) = socket.split();
        let mut base64_messages: Vec<String> = Vec::new();

        loop {
            let messages_string = unsafe { str::from_utf8_unchecked(&messages_bytes) };
            let messages: Vec<String> = messages_string
                .lines()
                .map(|line| line.to_string() + "\r\n")
                .collect();

            if messages.len() == 0 {
                break;
            }

            let args: Vec<&str> = messages[0].trim().split(' ').collect();
            match args[0] {
                "MSG" => {
                    let Ok(length) = args[3].parse::<usize>() else {
                        let tr_id = args[1];
                        let reply = format!("282 {tr_id}\r\n");

                        wr.write_all(reply.as_bytes())
                            .await
                            .expect("Could not send to client over socket");

                        trace!("S: {reply}");
                        return Ok(());
                    };

                    let length = messages[0].len() + length;

                    if length > messages_bytes.len() {
                        trace!("Fetching more message data...");

                        let mut buf = vec![0; 1664];
                        let Ok(received) = rd.read(&mut buf).await else {
                            return Err(ErrorCommand::Disconnect(
                                "Could not read from client".to_string(),
                            ));
                        };

                        if received == 0 {
                            return Err(ErrorCommand::Disconnect(
                                "Client disconnected".to_string(),
                            ));
                        }

                        let mut buf = buf[..received].to_vec();
                        messages_bytes.append(&mut buf);
                        continue;
                    }

                    let new_bytes = messages_bytes[..length].to_vec();
                    messages_bytes = messages_bytes[length..].to_vec();

                    let base64_message = URL_SAFE.encode(&new_bytes);
                    base64_messages.push(base64_message);
                }

                _ => {
                    let new_bytes = messages_bytes[..messages[0].len()].to_vec();
                    messages_bytes = messages_bytes[messages[0].len()..].to_vec();

                    let base64_message = URL_SAFE.encode(&new_bytes);
                    base64_messages.push(base64_message);
                }
            }
        }

        for base64_message in base64_messages {
            if self.session.is_none() {
                let mut handler = AuthenticationHandler::new(self.broadcast_tx.clone());
                handler.handle_command(&mut wr, base64_message).await?;

                self.protocol_version = handler.protocol_version;
                self.authenticated_user = handler.authenticated_user;
                self.session = handler.session;

                if let Some(session) = &self.session {
                    self.session_rx = Some(session.session_tx.subscribe());
                }
                continue;
            }

            let mut handler = SessionCommandHandler::new(
                self.broadcast_tx.clone(),
                self.session.clone().expect("Could not get session"),
                self.authenticated_user
                    .clone()
                    .expect("Could not get authenticated user"),
                self.protocol_version
                    .expect("Could not get protocol version"),
            );

            handler.handle_command(&mut wr, base64_message).await?;

            self.authenticated_user = Some(handler.authenticated_user);
            self.session = Some(handler.session);
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

        let message = URL_SAFE
            .decode(message)
            .expect("Could not decode base64 from session");

        let messages_string = unsafe { str::from_utf8_unchecked(&message) };
        let command = messages_string
            .lines()
            .next()
            .expect("Could not get command from session message")
            .to_string()
            + "\r\n";

        let args: Vec<&str> = command.trim().split(' ').collect();

        let principal = args[1];
        if principal
            == self
                .authenticated_user
                .as_ref()
                .expect("Could not get authenticated user")
                .email
        {
            return Ok(());
        }

        trace!("Thread {sender}: {command}");
        match args[0] {
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

    pub(crate) async fn send_bye_to_principals(&mut self, idling: bool) {
        let user_email = self
            .authenticated_user
            .as_ref()
            .expect("Could not get authenticated user")
            .email
            .clone();
        {
            let mut principals = self
                .session
                .as_ref()
                .expect("Could not get session")
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            let user_index = principals
                .iter()
                .position(|principal| principal.email == user_email)
                .expect("Could not find user among principals");

            principals.swap_remove(user_index);
        }

        let mut bye_command = Bye.generate(
            self.protocol_version
                .expect("Could not get protocol version"),
            self.authenticated_user
                .as_mut()
                .expect("Could not get authenticated user"),
            "",
        );

        if idling {
            bye_command = bye_command.replace("\r\n", " 1\r\n");
        }

        let message = Message::ToPrincipals {
            sender: user_email,
            message: URL_SAFE.encode(bye_command.as_bytes()),
        };

        if let Some(ref session) = self.session {
            session
                .session_tx
                .send(message)
                .expect("Could not send to session");
        }
    }
}
