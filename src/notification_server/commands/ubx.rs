use crate::errors::command_generation_error::CommandGenerationError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(user: &AuthenticatedUser) -> Result<String, CommandGenerationError> {
    let email = &user.email;
    let personal_message = &user
        .personal_message
        .as_ref()
        .ok_or(CommandGenerationError::CouldNotGetPersonalMessage)?;

    let length = personal_message.len();
    Ok(format!("UBX {email} {length}\r\n{personal_message}"))
}
