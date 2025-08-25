use crate::errors::command_error::CommandError;
use crate::{
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};

pub trait Command {
    async fn handle(
        &self,
        protocol_version: u32,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        command: &[u8],
    ) -> Result<Vec<String>, CommandError>;
}
