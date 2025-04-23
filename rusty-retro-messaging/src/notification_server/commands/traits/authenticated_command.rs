use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait AuthenticatedCommand {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String>;
}
