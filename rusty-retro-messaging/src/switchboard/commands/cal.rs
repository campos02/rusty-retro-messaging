use super::{
    invitation_error::InvitationError,
    rng::Rng,
    traits::{command::Command, thread_command::ThreadCommand},
};
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use core::str;
use tokio::sync::broadcast::{self, error::RecvError};

pub struct Cal {
    broadcast_tx: broadcast::Sender<Message>,
}

impl Cal {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Cal { broadcast_tx }
    }
}

impl Command for Cal {
    async fn handle(
        &self,
        protocol_version: usize,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        command: &Vec<u8>,
    ) -> Result<Vec<String>, ErrorCommand> {
        let command_string = unsafe { str::from_utf8_unchecked(&command) };
        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let tr_id = args[1];
        let email = args[2];

        {
            let principals = session
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            let user_index = principals
                .iter()
                .position(|principal| principal.email == email);

            if user_index.is_some() {
                return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
            }
        }

        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email.to_string(),
            message: "GetUserDetails".to_string(),
        };

        self.broadcast_tx
            .send(message)
            .expect("Could not send to broadcast");

        let mut principal_user;

        {
            let mut broadcast_rx = self.broadcast_tx.subscribe();
            loop {
                let message = match broadcast_rx.recv().await {
                    Ok(msg) => msg,
                    Err(err) => {
                        if let RecvError::Lagged(_) = err {
                            continue;
                        } else {
                            panic!("Could not receive from broadcast");
                        }
                    }
                };

                if let Message::UserDetails {
                    sender,
                    receiver: _,
                    authenticated_user,
                    protocol_version: _,
                } = message
                {
                    if sender == *email {
                        principal_user = authenticated_user;

                        if !broadcast_rx.is_empty() {
                            continue;
                        }
                        break;
                    }
                }
            }
        }

        if let Ok(presence) = principal_user
            .ok_or_else(|| InvitationError::PrincipalUserNotFound)
            .and_then(|authenticated_user| {
                authenticated_user
                    .presence
                    .ok_or_else(|| InvitationError::PrincipalOffline)
            })
        {
            if presence == "HDN" {
                return Err(ErrorCommand::Command(format!("217 {tr_id}\r\n")));
            }
        } else {
            return Err(ErrorCommand::Command(format!("217 {tr_id}\r\n")));
        }

        let rng = Rng {
            session_id: session.session_id.clone(),
            cki_string: session.cki_string.clone(),
        };

        let rng = rng.generate(protocol_version, user, tr_id);
        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email.to_string(),
            message: rng,
        };

        if self.broadcast_tx.send(message).is_err() {
            return Err(ErrorCommand::Command(format!("217 {tr_id}\r\n")));
        }

        Ok(vec![format!(
            "CAL {tr_id} RINGING {}\r\n",
            session.session_id
        )])
    }
}
