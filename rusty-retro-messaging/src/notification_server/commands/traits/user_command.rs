use crate::errors::command_error::CommandError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait UserCommand {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError>;
}
