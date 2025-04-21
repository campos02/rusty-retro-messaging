use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait Command {
    fn generate(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String;
}
