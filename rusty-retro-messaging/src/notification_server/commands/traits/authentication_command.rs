use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
};
use tokio::sync::broadcast;

pub trait AuthenticationCommand {
    fn handle(
        &self,
        protocol_version: usize,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &String,
    ) -> Result<(Vec<String>, AuthenticatedUser, broadcast::Receiver<Message>), ErrorCommand>;
}
