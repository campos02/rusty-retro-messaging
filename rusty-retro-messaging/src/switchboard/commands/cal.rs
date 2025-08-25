use super::rng;
use crate::errors::command_error::CommandError;
use crate::errors::invitation_error::InvitationError;
use crate::switchboard::commands::traits::command::Command;
use crate::{
    message::Message, models::transient::authenticated_user::AuthenticatedUser,
    switchboard::session::Session,
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
        protocol_version: u32,
        user: &mut AuthenticatedUser,
        session: &mut Session,
        command: &[u8],
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;

        let command_string = unsafe { str::from_utf8_unchecked(command) };
        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let email = args
            .get(2)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        {
            let principals =
                session
                    .principals
                    .lock()
                    .or(Err(CommandError::ReplyAndDisconnect(format!(
                        "500 {tr_id}\r\n"
                    ))))?;

            if principals.contains_key(&email) {
                return Err(CommandError::Reply(format!("215 {tr_id}\r\n")));
            }
        }

        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email.clone(),
            message: "GetUserDetails".to_string(),
        };

        self.broadcast_tx
            .send(message)
            .map_err(CommandError::CouldNotSendToBroadcast)?;

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
                            return Err(CommandError::CouldNotReceiveFromBroadcast(err));
                        }
                    }
                };

                if let Message::UserDetails {
                    sender,
                    receiver: _,
                    authenticated_user,
                    protocol_version: _,
                } = message
                    && sender == email
                {
                    principal_user = authenticated_user;
                    if !broadcast_rx.is_empty() {
                        continue;
                    }

                    break;
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
                return Err(CommandError::Reply(format!("217 {tr_id}\r\n")));
            }
        } else {
            return Err(CommandError::Reply(format!("217 {tr_id}\r\n")));
        }

        let rng = rng::generate(&session.session_id, &session.cki_string, user)
            .map_err(CommandError::CouldNotCreateRng)?;
        let message = Message::ToContact {
            sender: user.email.clone(),
            receiver: email,
            message: rng,
        };

        if self.broadcast_tx.send(message).is_err() {
            return Err(CommandError::Reply(format!("217 {tr_id}\r\n")));
        }

        Ok(vec![format!(
            "CAL {tr_id} RINGING {}\r\n",
            session.session_id
        )])
    }
}
