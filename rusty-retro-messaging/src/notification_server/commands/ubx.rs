use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(user: &AuthenticatedUser) -> Result<String, ErrorCommand> {
    let email = &user.email;
    let personal_message = &user.personal_message.as_ref().ok_or(ErrorCommand::Command(
        "Could not get authenticated user".to_string(),
    ))?;

    let length = personal_message.len();
    Ok(format!("UBX {email} {length}\r\n{personal_message}"))
}
