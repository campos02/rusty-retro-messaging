use crate::errors::command_generation_error::CommandGenerationError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(
    protocol_version: u32,
    user: &AuthenticatedUser,
) -> Result<String, CommandGenerationError> {
    let presence = &user
        .presence
        .as_ref()
        .ok_or(CommandGenerationError::NoPresence)?;

    let email = &user.email;
    let display_name = &user.display_name;
    let client_id = &user.client_id.ok_or(CommandGenerationError::NoClientId)?;

    Ok(
        if let Some(msn_object) = user.msn_object.as_ref()
            && protocol_version >= 9
        {
            format!("NLN {presence} {email} {display_name} {client_id} {msn_object}\r\n")
        } else {
            format!("NLN {presence} {email} {display_name} {client_id}\r\n")
        },
    )
}
