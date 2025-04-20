use super::command::Command;
use std::env;

pub struct Url;

impl Command for Url {
    fn handle(&mut self, protocol_version: usize, command: &String) -> Result<Vec<String>, String> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];
        let server_name = env::var("SERVER_NAME").expect("SERVER_NAME not set");

        Ok(vec![format!(
            "URL {tr_id} /url https://{server_name}/url 1\r\n"
        )])
    }
}
