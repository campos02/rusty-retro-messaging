use super::{
    joi::Joi,
    traits::{authentication_command::AuthenticationCommand, thread_command::ThreadCommand},
};
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::{authenticated_user::AuthenticatedUser, principal::Principal},
    switchboard::session::Session,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use core::str;
use tokio::sync::broadcast::{self, error::RecvError};

pub struct Ans;

impl AuthenticationCommand for Ans {
    async fn handle(
        &self,
        broadcast_tx: &broadcast::Sender<Message>,
        base64_command: &String,
    ) -> Result<(Vec<String>, usize, Session, AuthenticatedUser), ErrorCommand> {
        let bytes = URL_SAFE
            .decode(base64_command.clone())
            .expect("Could not decode client message from base64");

        let command = unsafe { str::from_utf8_unchecked(&bytes) };
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = args[1];
        let user_email = args[2];
        let cki_string = args[3];

        broadcast_tx
            .send(Message::GetSession(cki_string.to_string()))
            .expect("Could not send to broadcast");

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
                            panic!("Could not receive from broadcast");
                        }
                    }
                };

                if let Message::Session { key, value } = message {
                    if key == cki_string {
                        session = value;

                        if !broadcast_rx.is_empty() {
                            continue;
                        }
                        break;
                    }
                }
            }
        }

        let Some(session) = session else {
            return Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")));
        };

        let message = Message::ToContact {
            sender: user_email.to_string(),
            receiver: user_email.to_string(),
            message: "GetUserDetails".to_string(),
        };

        broadcast_tx
            .send(message)
            .expect("Could not send to broadcast");

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
                            panic!("Could not receive from broadcast");
                        }
                    }
                };

                if let Message::UserDetails {
                    sender,
                    receiver: _,
                    authenticated_user,
                    protocol_version,
                } = message
                {
                    if sender == user_email {
                        authenticated_user_result = authenticated_user;
                        protocol_version_result = protocol_version;

                        if !broadcast_rx.is_empty() {
                            continue;
                        }
                        break;
                    }
                }
            }
        }

        let mut authenticated_user: AuthenticatedUser =
            authenticated_user_result.expect("Could not get authenticated user");
        let protocol_version = protocol_version_result.expect("Could not get protocol version");

        let mut replies: Vec<String> = Vec::new();

        {
            let mut principals = session
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            let count = principals.len();
            let mut index = 1;

            for principal in principals.to_vec() {
                let email = principal.email;
                let display_name = principal.display_name;

                let mut iro_reply =
                    format!("IRO {tr_id} {index} {count} {email} {display_name}\r\n");

                if protocol_version >= 12 {
                    if let Some(client_id) = principal.client_id {
                        iro_reply = format!(
                            "IRO {tr_id} {index} {count} {email} {display_name} {client_id}\r\n"
                        );
                    }
                }

                replies.push(iro_reply);
                index += 1;
            }

            principals.push(Principal {
                email: user_email.to_string(),
                display_name: authenticated_user.display_name.clone(),
                client_id: authenticated_user.client_id.clone(),
            });
        }

        let joi = Joi.generate(protocol_version, &mut authenticated_user, tr_id);

        let message = Message::ToPrincipals {
            sender: user_email.to_string(),
            message: URL_SAFE.encode(joi.as_bytes()),
        };

        session
            .session_tx
            .send(message)
            .expect("Could not send to session");

        replies.push(format!("ANS {tr_id} OK\r\n"));
        Ok((replies, protocol_version, session, authenticated_user))
    }
}
