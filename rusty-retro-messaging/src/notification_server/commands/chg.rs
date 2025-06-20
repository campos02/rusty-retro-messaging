use super::{
    fln::Fln,
    traits::{thread_command::ThreadCommand, user_command::UserCommand},
};
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser,
};
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
    fn handle(
        &self,
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        // Validate presence
        match args[2] {
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

        user.presence = Some(args[2].to_string());

        let Ok(client_id) = args[3].parse() else {
            return Err(ErrorCommand::Command(format!("201 {tr_id}\r\n")));
        };

        user.client_id = Some(client_id);
        user.msn_object = if args.len() > 4 {
            Some(args[4].to_string())
        } else {
            None
        };

        for email in user.contacts.keys() {
            if let Some(contact) = user.contacts.get(email) {
                if user.blp == "BL" && !contact.in_allow_list {
                    continue;
                }

                if contact.in_block_list {
                    continue;
                }
            } else {
                if user.blp == "BL" {
                    continue;
                }
            }

            if args[2] != "HDN" {
                let nln_command = Chg::convert(&user, &command);
                let thread_message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: email.clone(),
                    message: nln_command,
                };

                self.broadcast_tx
                    .send(thread_message)
                    .expect("Could not send to broadcast");
            } else {
                let fln_command = Fln::convert(&user, &"".to_string());
                let message = Message::ToContact {
                    sender: user.email.clone(),
                    receiver: email.clone(),
                    message: fln_command,
                };

                self.broadcast_tx
                    .send(message)
                    .expect("Could not send to broadcast");

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
                    .expect("Could not send to broadcast");
            }
        }

        Ok(vec![command.to_string()])
    }
}

impl ThreadCommand for Chg {
    fn convert(user: &AuthenticatedUser, command: &str) -> String {
        let mut args = command.trim().split(' ');
        args.next();
        args.next();

        let presence = args.next().expect("CHG to be converted has no presence");
        let client_id = args.next().expect("CHG to be converted has no client id");
        let mut msn_object = String::from("");

        if let Some(object) = args.next() {
            let mut object = String::from(object);
            object.insert_str(0, " ");
            msn_object = object;
        }

        let email = &user.email;
        let display_name = &user.display_name;
        format!("NLN {presence} {email} {display_name} {client_id}{msn_object}\r\n")
    }
}
