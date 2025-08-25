use super::{joi, traits::authentication_command::AuthenticationCommand};
use crate::errors::command_error::CommandError;
use crate::{
    message::Message,
    models::transient::{authenticated_user::AuthenticatedUser, principal::Principal},
    switchboard::session::Session,
};
use core::str;
use std::sync::Arc;
use tokio::sync::broadcast::{self, error::RecvError};

pub struct Ans;

impl AuthenticationCommand for Ans {
    async fn handle(
        &self,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &[u8],
    ) -> Result<(Vec<String>, u32, Session, AuthenticatedUser), CommandError> {
        let command_string = unsafe { str::from_utf8_unchecked(command) };
        let args: Vec<&str> = command_string.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let user_email = args
            .get(2)
            .map(|str| Arc::new(str.to_string()))
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let cki_string = *args
            .get(3)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        broadcast_tx
            .send(Message::GetSession(Arc::new(cki_string.to_string())))
            .map_err(CommandError::CouldNotSendToBroadcast)?;

        let mut session;
        {
            let mut broadcast_rx = broadcast_tx.subscribe();
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

                if let Message::Session { key, value } = message
                    && *key == cki_string
                {
                    session = value;

                    if !broadcast_rx.is_empty() {
                        continue;
                    }
                    break;
                }
            }
        }

        let Some(session) = session else {
            return Err(CommandError::ReplyAndDisconnect(format!("911 {tr_id}\r\n")));
        };

        let message = Message::ToContact {
            sender: user_email.clone(),
            receiver: user_email.clone(),
            message: "GetUserDetails".to_string(),
        };

        broadcast_tx
            .send(message)
            .map_err(CommandError::CouldNotSendToBroadcast)?;

        let mut authenticated_user_result;
        let mut protocol_version_result;

        {
            let mut broadcast_rx = broadcast_tx.subscribe();
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
                    protocol_version,
                } = message
                    && sender == user_email
                {
                    authenticated_user_result = authenticated_user;
                    protocol_version_result = protocol_version;

                    if !broadcast_rx.is_empty() {
                        continue;
                    }

                    break;
                }
            }
        }

        let mut authenticated_user =
            authenticated_user_result.ok_or(CommandError::CouldNotGetAuthenticatedUser)?;

        let protocol_version =
            protocol_version_result.ok_or(CommandError::CouldNotGetProtocolVersion)?;

        let mut replies = Vec::new();
        {
            let mut principals =
                session
                    .principals
                    .lock()
                    .or(Err(CommandError::ReplyAndDisconnect(format!(
                        "500 {tr_id}\r\n"
                    ))))?;

            let count = principals.len();
            let mut index = 1;

            for principal in principals.values() {
                let email = &principal.email;
                let display_name = &principal.display_name;

                let iro_reply = if protocol_version >= 12
                    && let Some(client_id) = principal.client_id
                {
                    format!("IRO {tr_id} {index} {count} {email} {display_name} {client_id}\r\n")
                } else {
                    format!("IRO {tr_id} {index} {count} {email} {display_name}\r\n")
                };

                replies.push(iro_reply);
                index += 1;
            }

            principals.insert(
                user_email.clone(),
                Principal {
                    email: user_email.clone(),
                    display_name: authenticated_user.display_name.clone(),
                    client_id: authenticated_user.client_id,
                },
            );
        }

        let joi = joi::generate(protocol_version, &mut authenticated_user, tr_id);
        let message = Message::ToPrincipals {
            sender: user_email.clone(),
            message: joi.as_bytes().to_vec(),
        };

        session
            .session_tx
            .send(message)
            .or(Err(CommandError::ReplyAndDisconnect(format!(
                "500 {tr_id}\r\n"
            ))))?;

        replies.push(format!("ANS {tr_id} OK\r\n"));
        Ok((replies, protocol_version, session, authenticated_user))
    }
}
