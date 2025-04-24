use super::traits::broadcasted_command::BroadcastedCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub struct Bye;

impl BroadcastedCommand for Bye {
    fn generate(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        tr_id: &str,
    ) -> String {
        let _ = protocol_version;
        let _ = tr_id;
        let email = &user.email;

        format!("BYE {email}\r\n")
    }
}
