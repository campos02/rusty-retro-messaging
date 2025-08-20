use super::traits::command::Command;
use crate::error_command::ErrorCommand;
use std::env;

pub struct Url;

impl Command for Url {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;
        let server_name =
            env::var("SERVER_NAME").or(Err(ErrorCommand::Command(format!("500 {tr_id}\r\n"))))?;

        Ok(vec![format!(
            "URL {tr_id} /url https://{server_name}/url 1\r\n"
        )])
    }
}
