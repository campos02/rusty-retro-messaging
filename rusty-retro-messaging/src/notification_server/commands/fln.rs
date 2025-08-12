use crate::models::transient::authenticated_user::AuthenticatedUser;

pub(crate) fn convert(user: &AuthenticatedUser, command: &str) -> String {
    let _ = command;
    let email = &user.email;

    format!("FLN {email}\r\n")
}
