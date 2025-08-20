use super::fln;
use crate::notification_server::commands::traits::user_command::UserCommand;
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Chg {
    broadcast_tx: broadcast::Sender<Message>,
    first_chg: bool,
}

impl Chg {
    pub fn new(broadcast_tx: broadcast::Sender<Message>, first_chg: bool) -> Self {
        Chg {
            broadcast_tx,
            first_chg,
        }
    }
}

impl UserCommand for Chg {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let status = *args
            .get(2)
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        // Validate presence
        match status {
            "NLN" => (),
            "BSY" => (),
            "IDL" => (),
            "AWY" => (),
            "BRB" => (),
            "PHN" => (),
            "LUN" => (),
            "HDN" => (),
            _ => return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n"))),
        }

        let client_id = args
            .get(3)
            .unwrap_or(&"")
            .parse()
            .or(Err(ErrorCommand::Command(format!("201 {tr_id}\r\n"))))?;

        user.presence = Some(Arc::new(status.to_string()));
        user.client_id = Some(client_id);
        user.msn_object = if args.len() > 4 {
            Some(Arc::new(args[4].to_string()))
        } else {
            None
        };

        for email in user.contacts.keys() {
            if let Some(contact) = user.contacts.get(email) {
                if *user.blp == "BL" && !contact.in_allow_list {
                    continue;
                }

                if contact.in_block_list {
                    continue;
                }
            } else if *user.blp == "BL" {
                continue;
            }

            if status != "HDN" {
                let nln_command = convert(user, command);
                let thread_message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: email.clone(),
                    message: nln_command?,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to broadcast".to_string(),
                    )))?;
            } else {
                let fln_command = fln::convert(user, "");
                let message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: email.clone(),
                    message: fln_command,
                };

                self.broadcast_tx
                    .send(message)
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to broadcast".to_string(),
                    )))?;

                continue;
            }

            if self.first_chg {
                let thread_message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: email.clone(),
                    message: command.to_owned(),
                };

                self.broadcast_tx
                    .send(thread_message)
                    .or(Err(ErrorCommand::Disconnect(
                        "Could not send to broadcast".to_string(),
                    )))?;
            }
        }

        Ok(vec![command.to_string()])
    }
}

pub fn convert(user: &AuthenticatedUser, command: &str) -> Result<String, ErrorCommand> {
    let mut args = command.trim().split(' ');
    let presence = args.nth(2).ok_or(ErrorCommand::Command(
        "CHG to be converted has no presence".to_string(),
    ))?;

    let client_id = args.next().ok_or(ErrorCommand::Command(
        "CHG to be converted has no client id".to_string(),
    ))?;

    let mut msn_object = String::from("");

    if let Some(object) = args.next() {
        let mut object = String::from(object);
        object.insert(0, ' ');
        msn_object = object;
    }

    let email = &user.email;
    let display_name = &user.display_name;
    Ok(format!(
        "NLN {presence} {email} {display_name} {client_id}{msn_object}\r\n"
    ))
}
