use super::{invitation_error::InvitationError, rng};
use crate::switchboard::commands::traits::command::Command;
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use core::str;
use std::sync::Arc;
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
        command: &[u8],
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let command_string = unsafe { str::from_utf8_unchecked(command) };
        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let email = args
            .get(2)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(ErrorCommand::Command(format!("201 {tr_id}\r\n")))?;

        {
            let principals = session
                .principals
                .lock()
                .or(Err(ErrorCommand::Disconnect(format!("500 {tr_id}\r\n"))))?;

            if principals.contains_key(&email) {
                return Err(ErrorCommand::Command(format!("215 {tr_id}\r\n")));
            }
        }

        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email.clone(),
            message: "GetUserDetails".to_string(),
        };

        self.broadcast_tx
            .send(message)
            .or(Err(ErrorCommand::Disconnect(
                "Could not send to broadcast".to_string(),
            )))?;

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
                    if sender == email {
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
            .ok_or(InvitationError::PrincipalUserNotFound)
            .and_then(|authenticated_user| {
                authenticated_user
                    .presence
                    .ok_or(InvitationError::PrincipalOffline)
            })
        {
            if *presence == "HDN" {
                return Err(ErrorCommand::Command(format!("217 {tr_id}\r\n")));
            }
        } else {
            return Err(ErrorCommand::Command(format!("217 {tr_id}\r\n")));
        }

        let rng = rng::generate(&session.session_id, &session.cki_string, user, tr_id)?;
        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email,
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
