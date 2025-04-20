use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait Command {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String> {
        let _ = protocol_version;
        let _ = command;
        unimplemented!();
    }

    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let _ = user;
        let _ = command;
        unimplemented!();
    }
}
