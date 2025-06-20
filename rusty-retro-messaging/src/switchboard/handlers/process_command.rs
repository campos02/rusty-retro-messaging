use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
    switchboard::{
        commands::traits::{authentication_command::AuthenticationCommand, command::Command},
        session::Session,
    },
};
use log::{error, trace, warn};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::broadcast};

pub async fn process_authentication_command(
    broadcast_tx: &broadcast::Sender<Message>,
    wr: &mut WriteHalf<'_>,
    command: &impl AuthenticationCommand,
    message: &Vec<u8>,
) -> Result<(usize, Session, AuthenticatedUser), ErrorCommand> {
    match command.handle(broadcast_tx, message).await {
        Ok((responses, protocol_version, session, authenticated_user)) => {
            for reply in &responses {
                wr.write_all(reply.as_bytes())
                    .await
                    .expect("Could not send to client over socket");

                trace!("S: {reply}");
            }

            Ok((protocol_version, session, authenticated_user))
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

pub async fn process_session_command(
    protocol_version: usize,
    authenticated_user: &mut AuthenticatedUser,
    session: &mut Session,
    wr: &mut WriteHalf<'_>,
    command: &impl Command,
    message: &Vec<u8>,
) -> Result<Vec<String>, ErrorCommand> {
    match command
        .handle(protocol_version, authenticated_user, session, message)
        .await
    {
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
