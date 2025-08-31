use super::traits::user_command::UserCommand;
use crate::errors::command_error::CommandError;
use crate::{
    message::Message, models::transient::authenticated_user::AuthenticatedUser,
    switchboard::session::Session,
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use rand::distr::SampleString;
use rand_distr::Alphanumeric;
use std::collections::HashMap;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

pub struct Xfr {
    broadcast_tx: broadcast::Sender<Message>,
}

impl Xfr {
    pub fn new(broadcast_tx: broadcast::Sender<Message>) -> Self {
        Xfr { broadcast_tx }
    }
}

impl UserCommand for Xfr {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
        user: &mut AuthenticatedUser,
        version_number: &mut u32,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;
        let _ = version_number;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let server_type = *args
            .get(2)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        if server_type != "SB" {
            return Err(CommandError::Reply(format!("913 {tr_id}\r\n")));
        }

        if let Some(presence) = &user.presence {
            if **presence == "HDN" {
                return Err(CommandError::Reply(format!("913 {tr_id}\r\n")));
            }
        } else {
            return Err(CommandError::Reply(format!("913 {tr_id}\r\n")));
        }

        let switchboard_ip =
            env::var("SWITCHBOARD_IP").or(Err(CommandError::Reply(format!("500 {tr_id}\r\n"))))?;

        let cki_string = Arc::new(Alphanumeric.sample_string(&mut rand::rng(), 16));
        let (tx, _) = broadcast::channel::<Message>(16);
        let session_id = Arc::new(format!("{:08}", OsRng.next_u32()));

        let session = Session {
            session_id,
            cki_string: cki_string.clone(),
            session_tx: tx,
            principals: Arc::new(Mutex::new(HashMap::new())),
        };

        self.broadcast_tx
            .send(Message::SetSession {
                key: cki_string.clone(),
                value: session,
            })
            .map_err(CommandError::CouldNotSendToBroadcast)?;

        Ok(vec![format!(
            "XFR {tr_id} SB {switchboard_ip}:1864 CKI {cki_string}\r\n"
        )])
    }
}
