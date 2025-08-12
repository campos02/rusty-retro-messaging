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

pub async fn process_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    command: &impl Command,
    message: &str,
) -> Result<Vec<String>, ErrorCommand> {
    match command.handle(protocol_version, message).await {
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

pub async fn process_authentication_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    broadcast_tx: &broadcast::Sender<Message>,
    command: &impl AuthenticationCommand,
    message: &str,
) -> Result<(AuthenticatedUser, broadcast::Receiver<Message>), ErrorCommand> {
    match command
        .handle(protocol_version, broadcast_tx, message)
        .await
    {
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

pub async fn process_user_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    authenticated_user: &mut AuthenticatedUser,
    command: &impl UserCommand,
    message: &str,
) -> Result<Vec<String>, ErrorCommand> {
    match command
        .handle(protocol_version, message, authenticated_user)
        .await
    {
        Ok(responses) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                if reply.starts_with("XFR") {
                    let args: Vec<&str> = reply.split(' ').collect();
                    if args.len() < 6 {
                        error!("Reply doesn't have enough arguments: {reply}");
                        return Err(ErrorCommand::Command("Not enough arguments".to_string()));
                    }

                    trace!(
                        "S: {} {} {} {} {} xxxxx\r\n",
                        args[0], args[1], args[2], args[3], args[4]
                    );
                } else {
                    trace!("S: {reply}");
                }
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
