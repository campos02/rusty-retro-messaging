use super::traits::authentication_command::AuthenticationCommand;
use crate::error_command::ErrorCommand;
use crate::message::Message;
use crate::models::token::Token;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use crate::models::user::User;
use crate::schema::tokens::dsl::tokens;
use crate::schema::tokens::token;
use crate::schema::users::dsl::users;
use crate::schema::users::id;
use chrono::Utc;
use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, MysqlConnection, RunQueryDsl};
use diesel::{SelectableHelper, query_dsl::methods::SelectDsl};
use tokio::sync::broadcast;

pub struct UsrS {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl UsrS {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
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
    fn handle(
        &self,
        protocol_version: usize,
        broadcast_tx: &broadcast::Sender<Message>,
        command: &str,
    ) -> Result<(Vec<String>, AuthenticatedUser, broadcast::Receiver<Message>), ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(ErrorCommand::Disconnect(format!("500 {tr_id}\r\n")));
        };

        let Ok(result) = tokens
            .filter(token.eq(&args[4].trim()))
            .select(Token::as_select())
            .get_result(connection)
        else {
            return Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")));
        };

        if Utc::now().naive_utc() <= result.valid_until {
            let Ok(result) = users
                .filter(id.eq(&result.user_id))
                .select(User::as_select())
                .get_result(connection)
            else {
                return Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")));
            };

            let user_email = &result.email;

            broadcast_tx
                .send(Message::AddUser)
                .expect("Could not send to broadcast");

            let authenticated_user = AuthenticatedUser::new(user_email.clone());

            let thread_message = Message::ToContact {
                sender: user_email.to_string(),
                receiver: user_email.to_string(),
                message: "OUT OTH\r\n".to_string(),
            };

            broadcast_tx
                .send(thread_message)
                .expect("Could not send to broadcast");

            let (tx, _) = broadcast::channel::<Message>(16);
            broadcast_tx
                .send(Message::SetTx {
                    key: user_email.to_string(),
                    value: tx.clone(),
                })
                .expect("Could not send to broadcast");

            let contact_rx = tx.subscribe();
            let replies = vec![
                format!("USR {tr_id} OK {user_email} 1 0\r\n"),
                String::from("SBS 0 null\r\n"),
                Self::get_hotmail_options(&result),
            ];

            return Ok((replies, authenticated_user, contact_rx));
        }

        Err(ErrorCommand::Disconnect(format!("911 {tr_id}\r\n")))
    }
}
