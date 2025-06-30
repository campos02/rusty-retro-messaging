use super::contact_verification_error::ContactVerificationError;
use crate::notification_server::handlers::handle_authentication_command::handle_authentication_command;
use crate::notification_server::handlers::handle_thread_command::handle_thread_command;
use crate::notification_server::handlers::handle_user_command::handle_user_command;
use crate::notification_server::handlers::handle_ver::handle_ver;
use crate::receive_split::receive_split;
use crate::{
    error_command::ErrorCommand,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{fln::Fln, traits::thread_command::ThreadCommand},
    Message,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};
use tokio::{
    net::{tcp::WriteHalf, TcpStream},
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
        if self.authenticated_user.is_some() {
            tokio::select! {
                messages = receive_split(&mut rd) => {
                    self.handle_client_commands(&mut wr, messages?).await?
                }

                received = self.contact_rx.as_mut().expect("Could not receive from threads").recv() => {
                    self.handle_thread_commands(&mut wr, received.expect("Could not receive from threads")).await?
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
            if self.protocol_version.is_none() {
                self.protocol_version = handle_ver(wr, message).await?;
                continue;
            }

            if self.authenticated_user.is_none() {
                let (authenticated_user, contact_rx) = handle_authentication_command(
                    self.protocol_version
                        .expect("Could not get protocol version"),
                    &self.pool,
                    &self.broadcast_tx,
                    wr,
                    message,
                )
                .await?;

                self.authenticated_user = authenticated_user;
                self.contact_rx = contact_rx;
                continue;
            }

            handle_user_command(
                self.protocol_version
                    .expect("Could not get protocol version"),
                self.authenticated_user
                    .as_mut()
                    .expect("Could not get authenticated user"),
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
    ) -> Result<(), ErrorCommand> {
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
                .expect("Could not get protocol version"),
            self.authenticated_user
                .as_mut()
                .expect("Could not get authenticated user"),
            sender,
            &self.broadcast_tx,
            wr,
            message,
        )
        .await?;

        Ok(())
    }

    pub(crate) async fn send_fln_to_contacts(&mut self) {
        for email in self
            .authenticated_user
            .as_ref()
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
