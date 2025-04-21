use super::command::Command;
use crate::models::token::Token;
use crate::models::user::User;
use crate::schema::tokens::dsl::tokens;
use crate::schema::tokens::token;
use crate::schema::users::dsl::users;
use crate::schema::users::{email, id};
use chrono::Utc;
use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, MysqlConnection, RunQueryDsl};
use diesel::{SelectableHelper, query_dsl::methods::SelectDsl};

pub struct Usr {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    user_email: Option<String>,
}

impl Usr {
    pub fn new(pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Usr {
            pool,
            user_email: None,
        }
    }

    pub fn get_user_email(&self) -> Option<String> {
        self.user_email.clone()
    }

    fn get_hotmail_options(user: &User, msp_auth: String) -> String {
        let mut payload = String::from("MIME-Version: 1.0\r\n");
        let timestamp = Utc::now().timestamp();

        // Reversed in MSN for some reason
        let member_id_high = user.puid & 0xffffffff;
        let member_id_low = user.puid & 0xffffffff00000000;

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
        payload.push_str(format!("MSPAuth: {msp_auth}\r\n").as_str());
        payload.push_str("ClientIP: 24.111.111.111\r\n");
        payload.push_str("ClientPort: 60712\r\n");
        payload.push_str("ABCHMigrated: 1\r\n\r\n");

        let length = payload.as_bytes().len();
        format!("MSG Hotmail Hotmail {length}\r\n{payload}")
    }
}

impl Command for Usr {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let option = args[3];

        let Ok(connection) = &mut self.pool.get() else {
            return Err(format!("500 {tr_id}\r\n"));
        };

        if option == "I" {
            if users
                .filter(email.eq(args[4].trim()))
                .select(User::as_select())
                .get_result(connection)
                .is_err()
            {
                return Err(format!("911 {tr_id}\r\n"));
            }

            return Ok(vec![format!(
                "USR {tr_id} TWN S ct=1,rver=1,wp=FS_40SEC_0_COMPACT,lc=1,id=1\r\n"
            )]);
        }

        if option == "S" {
            let Ok(result) = tokens
                .filter(token.eq(&args[4].trim()))
                .select(Token::as_select())
                .get_result(connection)
            else {
                return Err(format!("911 {tr_id}\r\n"));
            };

            if Utc::now().naive_utc() <= result.valid_until {
                let msp_auth = result.token.clone().replace("t=", "");
                let Ok(result) = users
                    .filter(id.eq(&result.user_id))
                    .select(User::as_select())
                    .get_result(connection)
                else {
                    return Err(format!("911 {tr_id}\r\n"));
                };

                let result_email = &result.email;
                self.user_email = Some(result_email.clone());

                return Ok(vec![
                    format!("USR {tr_id} OK {result_email} 1 0\r\n"),
                    String::from("SBS 0 null\r\n"),
                    Usr::get_hotmail_options(&result, msp_auth),
                ]);
            }
        }

        Err(format!("911 {tr_id}\r\n"))
    }
}
