use super::traits::command_handler::CommandHandler;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::{cvr::Cvr, usr_i::UsrI, usr_s::UsrS},
};
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use log::warn;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub struct AuthenticationHandler {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    broadcast_tx: broadcast::Sender<Message>,
    pub protocol_version: usize,
    pub authenticated_user: Option<AuthenticatedUser>,
    pub contact_rx: Option<broadcast::Receiver<Message>>,
}

impl AuthenticationHandler {
    pub fn new(
        pool: Pool<ConnectionManager<MysqlConnection>>,
        broadcast_tx: broadcast::Sender<Message>,
        protocol_version: usize,
    ) -> Self {
        AuthenticationHandler {
            pool,
            broadcast_tx,
            protocol_version,
            authenticated_user: None,
            contact_rx: None,
        }
    }
}

impl CommandHandler for AuthenticationHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand> {
        let _ = sender;
        let args: Vec<&str> = command.trim().split(' ').collect();

        match args[0] {
            "CVR" => {
                Self::process_command(self.protocol_version, wr, &mut Cvr, &command).await?;
            }

            "USR" => match args[3] {
                "I" => {
                    let mut usr = UsrI::new(self.pool.clone());
                    Self::process_command(self.protocol_version, wr, &mut usr, &command).await?;
                }

                "S" => {
                    let mut usr = UsrS::new(self.pool.clone());
                    let (authenticated_user, contact_rx) = Self::process_authentication_command(
                        self.protocol_version,
                        wr,
                        &self.broadcast_tx,
                        &mut usr,
                        &command,
                    )
                    .await?;

                    self.authenticated_user = Some(authenticated_user);
                    self.contact_rx = Some(contact_rx);
                }

                _ => {
                    let tr_id = args[1];
                    let err = format!("911 {tr_id}\r\n");

                    wr.write_all(err.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    warn!("S: {err}");
                    return Err(ErrorCommand::Disconnect(err));
                }
            },

            _ => warn!("Unmatched command before authentication: {command}"),
        }

        Ok(())
    }
}
