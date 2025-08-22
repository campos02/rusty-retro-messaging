use crate::errors::command_error::CommandError;
use crate::{
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::traits::{authentication_command::AuthenticationCommand, command::Command},
        session::Session,
    },
};
use log::{error, trace, warn};
use std::error;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn process_authentication_command(
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: &impl AuthenticationCommand,
    message: &[u8],
) -> Result<Option<(usize, Session, AuthenticatedUser)>, Box<dyn error::Error + Send + Sync>> {
    match command.handle(broadcast_tx, message).await {
        Ok((responses, protocol_version, session, authenticated_user)) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes()).await?;
                trace!("S: {reply}");
            }

            Ok(Some((protocol_version, session, authenticated_user)))
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

pub async fn process_session_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    session: &mut Session,
    wr: &mut WriteHalf<'_>,
    command: &impl Command,
    message: &[u8],
) -> Result<Vec<String>, Box<dyn error::Error + Send + Sync>> {
    match command
        .handle(protocol_version, authenticated_user, session, message)
        .await
    {
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
