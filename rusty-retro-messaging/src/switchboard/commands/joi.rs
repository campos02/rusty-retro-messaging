use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn generate(protocol_version: usize, user: &mut AuthenticatedUser, tr_id: &str) -> String {
    let _ = tr_id;

    let user_email = &user.email;
    let user_display_name = &user.display_name;
    let client_id = &user.client_id;

    if protocol_version >= 12 {
        if let Some(client_id) = client_id {
            return format!("JOI {user_email} {user_display_name} {client_id}\r\n");
        }
    }

    format!("JOI {user_email} {user_display_name}\r\n")
}
