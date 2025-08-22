use crate::errors::server_error::ServerError;
use crate::errors::thread_command_error::ThreadCommandError;
use crate::notification_server::commands::fln;
use crate::notification_server::handlers::handle_authentication_command::handle_authentication_command;
use crate::notification_server::handlers::handle_thread_command::handle_thread_command;
use crate::notification_server::handlers::handle_user_command::handle_user_command;
use crate::notification_server::handlers::handle_ver::handle_ver;
use crate::receive_split::receive_split;
use crate::{Message, models::transient::authenticated_user::AuthenticatedUser};
use log::error;
use sqlx::{MySql, Pool};
use std::error;
use tokio::{
    net::{TcpStream, tcp::WriteHalf},
    sync::broadcast,
};

pub struct NotificationServer {
    pool: Pool<MySql>,
    broadcast_tx: broadcast::Sender<Message>,
    contact_rx: Option<broadcast::Receiver<Message>>,
    authenticated_user: Option<AuthenticatedUser>,
    protocol_version: Option<usize>,
}

impl NotificationServer {
    pub fn new(pool: Pool<MySql>, broadcast_tx: broadcast::Sender<Message>) -> Self {
        NotificationServer {
            pool,
            broadcast_tx: broadcast_tx.clone(),
            contact_rx: None,
            authenticated_user: None,
            protocol_version: None,
        }
    }

    pub async fn listen(
        &mut self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let (mut rd, mut wr) = socket.split();
        if self.authenticated_user.is_some() {
            tokio::select! {
                messages = receive_split(&mut rd) => {
                    match messages {
                        Ok(messages) => {
                            if let Err(error) = self.handle_client_commands(&mut wr, messages).await {
                                error!("{error}");
                                if let Some(user) = self.authenticated_user.as_ref() {
                                    self.broadcast_tx.send(Message::RemoveTx(user.email.clone()))?;
                                    self.send_fln_to_contacts().await?;
                                }

                                self.broadcast_tx.send(Message::RemoveUser)?;
                            }
                        }

                        Err(error) => {
                            error!("{error}");
                            self.broadcast_tx.send(Message::RemoveUser)?;
                        }
                    }
                }

                received = self.contact_rx.as_mut().ok_or(ThreadCommandError::ReceivingError)?.recv() => {
                    self.handle_thread_commands(&mut wr, received?).await?;
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
            if self.protocol_version.is_none() {
                self.protocol_version = Some(handle_ver(wr, message).await?);
                continue;
            }

            if self.authenticated_user.is_none() {
                let Some((authenticated_user, contact_rx)) = handle_authentication_command(
                    self.protocol_version
                        .ok_or(ServerError::CouldNotGetProtocolVersion)?,
                    &self.pool,
                    &self.broadcast_tx,
                    wr,
                    message,
                )
                .await?
                else {
                    continue;
                };

                self.authenticated_user = Some(authenticated_user);
                self.contact_rx = Some(contact_rx);
                continue;
            }

            handle_user_command(
                self.protocol_version
                    .ok_or(ServerError::CouldNotGetProtocolVersion)?,
                self.authenticated_user
                    .as_mut()
                    .ok_or(ServerError::CouldNotGetAuthenticatedUser)?,
                &self.pool,
                &self.broadcast_tx,
                wr,
                message,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_thread_commands(
        &mut self,
        wr: &mut WriteHalf<'_>,
        message: Message,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        let Message::ToContact {
            sender,
            receiver: _,
            message,
        } = message
        else {
            return Ok(());
        };

        handle_thread_command(
            self.protocol_version
                .ok_or(ThreadCommandError::CouldNotGetProtocolVersion)?,
            self.authenticated_user
                .as_mut()
                .ok_or(ThreadCommandError::CouldNotGetAuthenticatedUser)?,
            sender,
            &self.broadcast_tx,
            wr,
            message,
        )
        .await?;

        Ok(())
    }

    pub async fn send_fln_to_contacts(
        &mut self,
    ) -> Result<(), Box<dyn error::Error + Send + Sync>> {
        for email in self
            .authenticated_user
            .as_ref()
            .ok_or(ServerError::CouldNotGetAuthenticatedUser)?
            .contacts
            .keys()
        {
            let fln_command = fln::convert(
                self.authenticated_user
                    .as_ref()
                    .ok_or(ServerError::CouldNotGetAuthenticatedUser)?,
            );

            let message = Message::ToContact {
                sender: self
                    .authenticated_user
                    .as_ref()
                    .ok_or(ServerError::CouldNotGetAuthenticatedUser)?
                    .email
                    .clone(),
                receiver: email.clone(),
                message: fln_command,
            };

            self.broadcast_tx.send(message)?;
        }

        Ok(())
    }
}
