use crate::{
    error_command::ErrorCommand, models::transient::authenticated_user::AuthenticatedUser,
    switchboard::session::Session,
};

pub trait Command {
    async fn handle(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        base64_command: &String,
    ) -> Result<Vec<String>, ErrorCommand>;
}
