use super::traits::authentication_command::AuthenticationCommand;
use crate::errors::command_error::CommandError;
use crate::message::Message;
use crate::models::token::Token;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use chrono::Utc;
use sqlx::{MySql, Pool};
use tokio::sync::broadcast;

pub struct UsrS {
    pool: Pool<MySql>,
}

impl UsrS {
    pub fn new(pool: Pool<MySql>) -> Self {
        UsrS { pool }
    }

    fn get_hotmail_options(user: &User) -> String {
        let mut payload = String::from("MIME-Version: 1.0\r\n");
        let timestamp = Utc::now().timestamp();

        // High and low 32 bits of PUID
        let member_id_high = ((user.puid & 0xffffffff00000000) >> 32) as u32;
        let member_id_low = (user.puid & 0xffffffff) as u32;

        payload.push_str("Content-Type: text/x-msmsgsprofile; charset=UTF-8\r\n");
        payload.push_str(format!("LoginTime: {timestamp}\r\n").as_str());
        payload.push_str("EmailEnabled: 0\r\n");
        payload.push_str(format!("MemberIdHigh: {member_id_high}\r\n").as_str());
        payload.push_str(format!("MemberIdLow: {member_id_low}\r\n").as_str());
        payload.push_str("lang_preference: 1036\r\n");
        payload.push_str("preferredEmail: \r\n");
        payload.push_str("country: \r\n");
        payload.push_str("PostalCode: \r\n");
        payload.push_str("Gender: \r\n");
        payload.push_str("Kid: \r\n");
        payload.push_str("Age: \r\n");
        payload.push_str("BDayPre: \r\n");
        payload.push_str("Birthday: \r\n");
        payload.push_str("Wallet: \r\n");
        payload.push_str("Flags: 1027\r\n");
        payload.push_str("sid: 507\r\n");
        payload.push_str("MSPAuth: \r\n");
        payload.push_str("ClientIP: 24.111.111.111\r\n");
        payload.push_str("ClientPort: 60712\r\n");
        payload.push_str("ABCHMigrated: 1\r\n\r\n");

        let length = payload.len();
        format!("MSG Hotmail Hotmail {length}\r\n{payload}")
    }
}

impl AuthenticationCommand for UsrS {
    async fn handle(
        &self,
        protocol_version: u32,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &str,
    ) -> Result<(Vec<String>, AuthenticatedUser, broadcast::Receiver<Message>), CommandError> {
        let _ = protocol_version;
        let args: Vec<&str> = command.trim().split(' ').collect();

        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let email = *args
            .get(4)
            .ok_or(CommandError::Reply(format!("201 {tr_id}\r\n")))?;

        let token = sqlx::query_as!(
            Token,
            "SELECT id, token, valid_until, user_id FROM tokens WHERE token = ? LIMIT 1",
            email.trim()
        )
        .fetch_one(&self.pool)
        .await
        .or(Err(CommandError::Reply(format!("911 {tr_id}\r\n"))))?;

        if Utc::now().naive_utc() <= token.valid_until {
            let database_user = sqlx::query_as!(
                User,
                "SELECT id, email, password, display_name, puid, guid, gtc, blp 
                FROM users WHERE id = ? LIMIT 1",
                token.user_id
            )
            .fetch_one(&self.pool)
            .await
            .or(Err(CommandError::Reply(format!("911 {tr_id}\r\n"))))?;

            broadcast_tx
                .send(Message::AddUser)
                .map_err(CommandError::CouldNotSendToBroadcast)?;

            let authenticated_user = AuthenticatedUser::new(database_user.email.clone());
            let thread_message = Message::ToContact {
                sender: database_user.email.clone(),
                receiver: database_user.email.clone(),
                message: "OUT OTH\r\n".to_string(),
            };

            broadcast_tx
                .send(thread_message)
                .map_err(CommandError::CouldNotSendToBroadcast)?;

            let (tx, _) = broadcast::channel::<Message>(16);
            broadcast_tx
                .send(Message::SetTx {
                    key: database_user.email.clone(),
                    value: tx.clone(),
                })
                .map_err(CommandError::CouldNotSendToBroadcast)?;

            let contact_rx = tx.subscribe();
            let mut replies = vec![
                if protocol_version >= 10 {
                    format!("USR {tr_id} OK {} 1 0\r\n", database_user.email)
                } else {
                    format!(
                        "USR {tr_id} OK {} {} 1 0\r\n",
                        database_user.email, database_user.display_name
                    )
                },
                Self::get_hotmail_options(&database_user),
            ];

            if protocol_version >= 10 {
                replies.insert(1, String::from("SBS 0 null\r\n"));
            }

            return Ok((replies, authenticated_user, contact_rx));
        }

        Err(CommandError::ReplyAndDisconnect(format!("911 {tr_id}\r\n")))
    }
}
