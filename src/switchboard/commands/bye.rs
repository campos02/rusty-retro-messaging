use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn generate(protocol_version: u32, user: &mut AuthenticatedUser, tr_id: &str) -> String {
    let _ = protocol_version;
    let _ = tr_id;

    let email = &user.email;
    format!("BYE {email}\r\n")
}
