use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use std::env;

pub fn generate(
    session_id: &str,
    cki_string: &str,
    user: &mut AuthenticatedUser,
    tr_id: &str,
) -> Result<String, ErrorCommand> {
    let switchboard_ip =
        env::var("SWITCHBOARD_IP").or(Err(ErrorCommand::Command(format!("500 {tr_id}\r\n"))))?;

    let email = &user.email;
    let display_name = &user.display_name;

    Ok(format!(
        "RNG {session_id} {switchboard_ip}:1864 CKI {cki_string} {email} {display_name}\r\n"
    ))
}
