use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(user: &AuthenticatedUser) -> String {
    let email = &user.email;
    format!("FLN {email}\r\n")
}
