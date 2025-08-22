use crate::errors::command_error::CommandError;
use crate::{message::Message, models::transient::authenticated_user::AuthenticatedUser};
use tokio::sync::broadcast;

pub trait AuthenticationCommand {
    async fn handle(
        &self,
        protocol_version: usize,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &str,
    ) -> Result<(Vec<String>, AuthenticatedUser, broadcast::Receiver<Message>), CommandError>;
}
