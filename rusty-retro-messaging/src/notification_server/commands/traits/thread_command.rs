use crate::models::transient::authenticated_user::AuthenticatedUser;

pub trait ThreadCommand {
    fn convert(user: &AuthenticatedUser, command: &String) -> String;
}
