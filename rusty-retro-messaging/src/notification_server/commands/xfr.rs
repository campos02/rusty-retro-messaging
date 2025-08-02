use super::traits::user_command::UserCommand;
use crate::{
    error_command::ErrorCommand, message::Message,
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use rand::distr::SampleString;
use rand_distr::Alphanumeric;
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
        protocol_version: usize,
        command: &str,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let server_type = args[2];

        if server_type != "SB" {
            return Err(ErrorCommand::Command(format!("913 {tr_id}\r\n")));
        }

        if let Some(presence) = &user.presence {
            if presence == "HDN" {
                return Err(ErrorCommand::Command(format!("913 {tr_id}\r\n")));
            }
        } else {
            return Err(ErrorCommand::Command(format!("913 {tr_id}\r\n")));
        }

        let switchboard_ip = env::var("SWITCHBOARD_IP").expect("SWITCHBOARD_IP not set");
        let cki_string = Alphanumeric.sample_string(&mut rand::rng(), 16);

        let (tx, _) = broadcast::channel::<Message>(16);
        let session_id = format!("{:08}", OsRng.next_u32());

        let session = Session {
            session_id,
            cki_string: cki_string.clone(),
            session_tx: tx,
            principals: Arc::new(Mutex::new(Vec::new())),
        };

        self.broadcast_tx
            .send(Message::SetSession {
                key: cki_string.clone(),
                value: session,
            })
            .expect("Could not send to broadcast");

        Ok(vec![format!(
            "XFR {tr_id} SB {switchboard_ip}:1864 CKI {cki_string}\r\n"
        )])
    }
}
