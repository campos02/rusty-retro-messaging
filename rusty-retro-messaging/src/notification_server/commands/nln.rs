use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub fn convert(user: &AuthenticatedUser) -> Result<String, ErrorCommand> {
    let presence = &user.presence.as_ref().ok_or(ErrorCommand::Command(
        "User has no presence set".to_string(),
    ))?;

    let email = &user.email;
    let display_name = &user.display_name;
    let client_id = &user.client_id.ok_or(ErrorCommand::Command(
        "User has no client id set".to_string(),
    ))?;

    Ok(if let Some(msn_object) = user.msn_object.as_ref() {
        format!("NLN {presence} {email} {display_name} {client_id} {msn_object}\r\n")
    } else {
        format!("NLN {presence} {email} {display_name} {client_id}\r\n")
    })
}
