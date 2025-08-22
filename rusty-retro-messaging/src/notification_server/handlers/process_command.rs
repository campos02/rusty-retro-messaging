use crate::errors::command_error::CommandError;
use crate::{
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    notification_server::commands::traits::{
        authentication_command::AuthenticationCommand, command::Command, user_command::UserCommand,
    },
};
use log::{error, trace, warn};
use std::error;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn process_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    command: &impl Command,
    message: &str,
) -> Result<Vec<String>, Box<dyn error::Error + Send + Sync>> {
    match command.handle(protocol_version, message).await {
        Ok(responses) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes()).await?;
                trace!("S: {reply}");
            }

            Ok(responses)
        }

        Err(CommandError::Reply(err)) => {
            wr.write_all(err.as_bytes()).await?;
            warn!("S: {err}");
            Ok(vec![])
        }

        Err(CommandError::ReplyAndDisconnect(err)) => {
            wr.write_all(err.as_bytes()).await?;
            error!("S: {err}");
            Err(CommandError::ReplyAndDisconnect(err).into())
        }

        Err(err) => Err(err.into()),
    }
}

pub async fn process_authentication_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    broadcast_tx: &broadcast::Sender<Message>,
    command: &impl AuthenticationCommand,
    message: &str,
) -> Result<
    Option<(AuthenticatedUser, broadcast::Receiver<Message>)>,
    Box<dyn error::Error + Send + Sync>,
> {
    match command
        .handle(protocol_version, broadcast_tx, message)
        .await
    {
        Ok((responses, authenticated_user, contact_rx)) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes()).await?;
                trace!("S: {reply}");
            }

            Ok(Some((authenticated_user, contact_rx)))
        }

        Err(CommandError::Reply(err)) => {
            wr.write_all(err.as_bytes()).await?;
            warn!("S: {err}");
            Ok(None)
        }

        Err(CommandError::ReplyAndDisconnect(err)) => {
            wr.write_all(err.as_bytes()).await?;
            error!("S: {err}");
            Err(CommandError::ReplyAndDisconnect(err).into())
        }

        Err(err) => Err(err.into()),
    }
}

pub async fn process_user_command(
    protocol_version: usize,
    wr: &mut WriteHalf<'_>,
    authenticated_user: &mut AuthenticatedUser,
    command: &impl UserCommand,
    message: &str,
) -> Result<Vec<String>, Box<dyn error::Error + Send + Sync>> {
    match command
        .handle(protocol_version, message, authenticated_user)
        .await
    {
        Ok(responses) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes()).await?;
                if reply.starts_with("XFR") {
                    let args: Vec<&str> = reply.split(' ').collect();
                    if args.len() < 5 {
                        return Err(CommandError::NotEnoughArguments.into());
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

        Err(CommandError::Reply(err)) => {
            wr.write_all(err.as_bytes()).await?;
            warn!("S: {err}");
            Ok(vec![])
        }

        Err(CommandError::ReplyAndDisconnect(err)) => {
            wr.write_all(err.as_bytes()).await?;
            error!("S: {err}");
            Err(CommandError::ReplyAndDisconnect(err).into())
        }

        Err(err) => Err(err.into()),
    }
}
