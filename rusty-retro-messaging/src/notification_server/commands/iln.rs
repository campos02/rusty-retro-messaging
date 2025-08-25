use crate::errors::command_generation_error::CommandGenerationError;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(
    protocol_version: u32,
    user: &AuthenticatedUser,
    command: &str,
) -> Result<String, CommandGenerationError> {
    let args: Vec<&str> = command.trim().split(' ').collect();
    let tr_id = *args.get(1).ok_or(CommandGenerationError::NoTrId)?;

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
            format!("ILN {tr_id} {presence} {email} {display_name} {client_id} {msn_object}\r\n")
        } else {
            format!("ILN {tr_id} {presence} {email} {display_name} {client_id}\r\n")
        },
    )
}
