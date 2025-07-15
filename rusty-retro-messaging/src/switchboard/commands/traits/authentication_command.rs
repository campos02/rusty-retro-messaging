use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use tokio::sync::broadcast;

pub trait AuthenticationCommand {
    async fn handle(
        &self,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &[u8],
    ) -> Result<(Vec<String>, usize, Session, AuthenticatedUser), ErrorCommand>;
}
