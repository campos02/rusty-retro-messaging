use super::traits::authentication_command::AuthenticationCommand;
use crate::{
    error_command::ErrorCommand,
    message::Message,
    models::transient::{authenticated_user::AuthenticatedUser, principal::Principal},
    switchboard::session::Session,
};
use core::str;
use tokio::sync::broadcast::{self, error::RecvError};

pub struct Usr;

impl AuthenticationCommand for Usr {
    async fn handle(
        &self,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &[u8],
    ) -> Result<(Vec<String>, usize, Session, AuthenticatedUser), ErrorCommand> {
        let command_string = unsafe { str::from_utf8_unchecked(command) };
        let args: Vec<&str> = command_string.trim().split(' ').collect();

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

        let authenticated_user: AuthenticatedUser =
            authenticated_user_result.expect("Could not get authenticated user");
        let protocol_version = protocol_version_result.expect("Could not get protocol version");

        let user_email = &authenticated_user.email;
        let user_display_name = &authenticated_user.display_name;

        {
            let mut principals = session
                .principals
                .lock()
                .expect("Could not get principals, mutex poisoned");

            principals.push(Principal {
                email: user_email.to_string(),
                display_name: authenticated_user.display_name.clone(),
                client_id: authenticated_user.client_id,
            });
        }

        Ok((
            vec![format!(
                "USR {tr_id} OK {user_email} {user_display_name}\r\n"
            )],
            protocol_version,
            session,
            authenticated_user,
        ))
    }
}
