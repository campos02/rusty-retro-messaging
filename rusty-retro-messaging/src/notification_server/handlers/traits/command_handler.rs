use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::traits::{
        authenticated_command::AuthenticatedCommand, authentication_command::AuthenticationCommand,
        command::Command,
    },
};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub trait CommandHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand>;

    async fn run_command(
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

                    println!("S: {reply}");
                }
                Ok(responses)
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }

    async fn run_authentication_command(
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

                    println!("S: {reply}");
                }
                Ok((authenticated_user, contact_rx))
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }

    async fn run_authenticated_command(
        protocol_version: usize,
        authenticated_user: &mut AuthenticatedUser,
        wr: &mut WriteHalf<'_>,
        command: &mut impl AuthenticatedCommand,
        message: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        match command.handle(protocol_version, &message, authenticated_user) {
            Ok(responses) => {
                for reply in &responses {
                    wr.write_all(reply.as_bytes())
                        .await
                        .expect("Could not send to client over socket");

                    println!("S: {reply}");
                }
                Ok(responses)
            }

            Err(ErrorCommand::Command(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Command(err))
            }

            Err(ErrorCommand::Disconnect(err)) => {
                wr.write_all(err.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                println!("S: {err}");
                Err(ErrorCommand::Disconnect(err))
            }
        }
    }
}
