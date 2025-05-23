use crate::{
    error_command::ErrorCommand, models::transient::authenticated_user::AuthenticatedUser,
};

pub trait UserCommand {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand>;
}
