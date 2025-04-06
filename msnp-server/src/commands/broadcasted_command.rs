use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait BroadcastedCommand {
    fn convert(user: &AuthenticatedUser, command: &String) -> String;
}
