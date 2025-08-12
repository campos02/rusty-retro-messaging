use crate::error_command::ErrorCommand;
use crate::models::transient::authenticated_user::AuthenticatedUser;

pub(crate) fn convert(user: &AuthenticatedUser, command: &str) -> Result<String, ErrorCommand> {
    let args: Vec<&str> = command.trim().split(' ').collect();

    let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
    let presence = &user.presence.as_ref().expect("User has no presence set");
    let email = &user.email;
    let display_name = &user.display_name;
    let client_id = &user.client_id.expect("User has no client id set");

    Ok(if let Some(msn_object) = user.msn_object.as_ref() {
        format!("ILN {tr_id} {presence} {email} {display_name} {client_id} {msn_object}\r\n")
    } else {
        format!("ILN {tr_id} {presence} {email} {display_name} {client_id}\r\n")
    })
}
