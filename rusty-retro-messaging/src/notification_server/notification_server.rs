use super::contact_verification_error::ContactVerificationError;
use crate::{
    Message,
    error_command::ErrorCommand,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::{
        commands::{fln::Fln, traits::thread_command::ThreadCommand},
        handlers::{
            authentication_handler::AuthenticationHandler, command_handler::CommandHandler,
            thread_command_handler::ThreadCommandHandler, user_command_handler::UserCommandHandler,
            ver_handler::VerHandler,
        },
    },
};
use core::str;
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use log::trace;
use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast,
};

pub struct NotificationServer {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    pub broadcast_tx: broadcast::Sender<Message>,
    contact_rx: Option<broadcast::Receiver<Message>>,
    pub authenticated_user: Option<AuthenticatedUser>,
    protocol_version: Option<usize>,
}

impl NotificationServer {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
    ) -> Self {
        NotificationServer {
            pool,
            broadcast_tx: broadcast_tx.clone(),
            contact_rx: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }

    pub async fn listen(&mut self, socket: &mut TcpStream) -> Result<(), ErrorCommand> {
        let (mut rd, mut wr) = socket.split();
        let mut buf = vec![0; 1664];

        if self.authenticated_user.is_some() {
            tokio::select! {
                received = rd.read(&mut buf) => {
                    let Ok(received) = received else {
                        return Err(ErrorCommand::Disconnect("Could not read from client".to_string()));
                    };

                    if received == 0 {
                        return Err(ErrorCommand::Disconnect("Client disconnected".to_string()));
                    }

                    let messages = str::from_utf8(&buf[..received]).expect("NS message contained invalid UTF-8").to_string();
                    self.handle_client_commands(&mut wr, messages).await?
                }

                received = self.contact_rx.as_mut().expect("Could not receive from threads").recv() => {
                    self.handle_thread_commands(&mut wr, received.expect("Could not receive from threads")).await?
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

                    let messages = str::from_utf8(&buf[..received]).expect("NS message contained invalid UTF-8").to_string();
                    self.handle_client_commands(&mut wr, messages).await?
                }
            }
        }

        Ok(())
    }

    async fn handle_client_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        messages: String,
    ) -> Result<(), ErrorCommand> {
        let mut messages: Vec<String> = messages.lines().map(|line| line.to_string()).collect();

        // Hack to concat payloads and payload commands
        for i in 0..messages.len() - 1 {
            let args: Vec<&str> = messages[i].trim().split(' ').collect();

            match args[0] {
                "UUX" => {
                    let Ok(length) = args[2].parse::<usize>() else {
                        return Err(ErrorCommand::Disconnect(
                            "Client sent invalid length".to_string(),
                        ));
                    };

                    let length = messages[i].len() + length;

                    messages[i] = messages[i].clone() + "\r\n" + messages[i + 1].as_str();
                    let next = messages[i].split_off(length + "\r\n".len());

                    if next != "" {
                        messages[i + 1] = next;
                    } else {
                        messages.remove(i + 1);
                    }
                }

                _ => (),
            }
        }

        let messages: Vec<String> = messages
            .iter()
            .map(|msg| msg.to_string() + "\r\n")
            .collect();

        for message in messages {
            trace!("C: {message}");

            if self.protocol_version.is_none() {
                let mut handler = VerHandler::new();
                handler.handle_command("".to_string(), wr, message).await?;

                self.protocol_version = handler.protocol_version;
                continue;
            }

            if self.authenticated_user.is_none() {
                let mut handler = AuthenticationHandler::new(
                    self.pool.clone(),
                    self.broadcast_tx.clone(),
                    self.protocol_version
                        .expect("Could not get protocol version"),
                );

                handler.handle_command("".to_string(), wr, message).await?;

                self.authenticated_user = handler.authenticated_user;
                self.contact_rx = handler.contact_rx;
                continue;
            }

            let mut handler = UserCommandHandler::new(
                self.pool.clone(),
                self.broadcast_tx.clone(),
                self.authenticated_user
                    .clone()
                    .expect("Could not get authenticated user"),
                self.protocol_version
                    .expect("Could not get protocol version"),
            );

            handler.handle_command("".to_string(), wr, message).await?;
            self.authenticated_user = Some(handler.authenticated_user);
        }

        Ok(())
    }

    async fn handle_thread_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        message: Message,
    ) -> Result<(), ErrorCommand> {
        let Message::ToContact {
            sender,
            receiver: _,
            message,
        } = message
        else {
            return Ok(());
        };

        trace!("Thread {sender}: {message}");
        let mut handler = ThreadCommandHandler::new(
            self.broadcast_tx.clone(),
            self.authenticated_user
                .clone()
                .expect("Could not get authenticated user"),
            self.protocol_version
                .expect("Could not get protocol version"),
        );

        handler.handle_command(sender, wr, message).await?;
        self.authenticated_user = Some(handler.authenticated_user);

        Ok(())
    }

    pub(crate) async fn send_fln_to_contacts(&mut self) {
        for email in self
            .authenticated_user
            .clone()
            .expect("Could not get authenticated user")
            .contacts
            .keys()
        {
            let fln_command = Fln::convert(
                &self
                    .authenticated_user
                    .as_ref()
                    .expect("Could not get authenticated user"),
                &"".to_string(),
            );

            let message = Message::ToContact {
                sender: self
                    .authenticated_user
                    .as_ref()
                    .expect("Could not get authenticated user")
                    .email
                    .clone(),
                receiver: email.clone(),
                message: fln_command,
            };

            self.broadcast_tx
                .send(message)
                .expect("Could not send to broadcast");
        }
    }

    pub(crate) fn verify_contact(
        authenticated_user: &AuthenticatedUser,
        email: &String,
    ) -> Result<(), ContactVerificationError> {
        if let Some(contact) = authenticated_user.contacts.get(email) {
            if authenticated_user.blp == "BL" && !contact.in_allow_list {
                return Err(ContactVerificationError::ContactNotInAllowList);
            }

            if contact.in_block_list {
                return Err(ContactVerificationError::ContactInBlockList);
            }

            if let Some(presence) = &authenticated_user.presence {
                if presence == "HDN" {
                    return Err(ContactVerificationError::UserAppearingOffline);
                }
            } else {
                return Err(ContactVerificationError::UserAppearingOffline);
            }
        } else {
            if authenticated_user.blp == "BL" && *email != authenticated_user.email {
                return Err(ContactVerificationError::ContactNotInAllowList);
            }
        }

        Ok(())
    }
}
