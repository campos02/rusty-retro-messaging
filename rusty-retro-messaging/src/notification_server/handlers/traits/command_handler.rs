use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::traits::{
        authentication_command::AuthenticationCommand, command::Command, user_command::UserCommand,
    },
};
use log::{error, trace, warn};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub trait CommandHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand>;

    async fn process_command(
        protocol_version: usize,
        wr: &mut WriteHalf<'_>,
        command: &mut impl Command,
        message: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        match command.handle(protocol_version, &message) {
            Ok(responses) => {
                for reply in &responses {
                    wr.write_all(reply.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    trace!("S: {reply}");
                }
                Ok(responses)
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                error!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }

    async fn process_authentication_command(
        protocol_version: usize,
        wr: &mut WriteHalf<'_>,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &mut impl AuthenticationCommand,
        message: &String,
    ) -> Result<(AuthenticatedUser, broadcast::Receiver<Message>), ErrorCommand> {
        match command.handle(protocol_version, broadcast_tx, &message) {
            Ok((responses, authenticated_user, contact_rx)) => {
                for reply in &responses {
                    wr.write_all(reply.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    trace!("S: {reply}");
                }
                Ok((authenticated_user, contact_rx))
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                error!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }

    async fn process_user_command(
        protocol_version: usize,
        wr: &mut WriteHalf<'_>,
        authenticated_user: &mut AuthenticatedUser,
        command: &mut impl UserCommand,
        message: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        match command.handle(protocol_version, &message, authenticated_user) {
            Ok(responses) => {
                for reply in &responses {
                    wr.write_all(reply.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    trace!("S: {reply}");
                }
                Ok(responses)
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                warn!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                error!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }
}
