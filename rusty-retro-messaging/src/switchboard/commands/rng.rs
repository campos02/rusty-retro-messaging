use crate::models::transient::authenticated_user::AuthenticatedUser;
use std::env;

pub(crate) fn generate(
    session_id: &str,
    cki_string: &str,
    protocol_version: usize,
    user: &mut AuthenticatedUser,
    tr_id: &str,
) -> String {
    let _ = protocol_version;
    let _ = tr_id;

    let switchboard_ip = env::var("SWITCHBOARD_IP").expect("SWITCHBOARD_IP not set");
    let email = &user.email;
    let display_name = &user.display_name;

    format!("RNG {session_id} {switchboard_ip}:1864 CKI {cki_string} {email} {display_name}\r\n")
}
