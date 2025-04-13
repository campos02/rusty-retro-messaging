use super::command::Command;
use crate::models::transient::authenticated_user::AuthenticatedUser;
use rand::distr::SampleString;
use rand_distr::Alphanumeric;
use std::env;

pub struct Xfr;

impl Command for Xfr {
    fn handle_with_authenticated_user(
        &mut self,
        command: &String,
        user: &mut AuthenticatedUser,
    ) -> Result<Vec<String>, String> {
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let server_type = args[2];

        if server_type != "SB" {
            return Err(format!("913 {tr_id}\r\n"));
        }

        if let Some(presence) = &user.presence {
            if presence == "HDN" {
                return Err(format!("913 {tr_id}\r\n"));
            }
        } else {
            return Err(format!("913 {tr_id}\r\n"));
        }

        let switchboard_ip = env::var("SWITCHBOARD_IP").expect("SWITCHBOARD_IP not set");
        let switchboard_port = env::var("SWITCHBOARD_PORT").expect("SWITCHBOARD_PORT not set");
        let cki_string = Alphanumeric.sample_string(&mut rand::rng(), 16);

        Ok(vec![format!(
            "XFR {tr_id} SB {switchboard_ip}:{switchboard_port} CKI {cki_string}\r\n"
        )])
    }
}
