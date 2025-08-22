use crate::errors::command_generation_error::CommandGenerationError;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use std::env;

pub fn generate(
    session_id: &str,
    cki_string: &str,
    user: &mut AuthenticatedUser,
) -> Result<String, CommandGenerationError> {
    let switchboard_ip =
        env::var("SWITCHBOARD_IP").or(Err(CommandGenerationError::SwitchboardIpNotSet))?;

    let email = &user.email;
    let display_name = &user.display_name;

    Ok(format!(
        "RNG {session_id} {switchboard_ip}:1864 CKI {cki_string} {email} {display_name}\r\n"
    ))
}
