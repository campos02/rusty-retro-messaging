use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait UserCommand {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, CommandError>;
}
