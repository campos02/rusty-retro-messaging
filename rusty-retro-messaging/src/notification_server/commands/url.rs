use super::traits::command::Command;
use crate::errors::command_error::CommandError;
use std::env;

pub struct Url;

impl Command for Url {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;
        let server_name =
            env::var("SERVER_NAME").or(Err(CommandError::Reply(format!("500 {tr_id}\r\n"))))?;

        Ok(vec![format!(
            "URL {tr_id} /url https://{server_name}/url 1\r\n"
        )])
    }
}
