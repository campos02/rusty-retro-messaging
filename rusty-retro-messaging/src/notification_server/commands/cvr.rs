use super::traits::command::Command;
use crate::errors::command_error::CommandError;

pub struct Cvr;

impl Command for Cvr {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;

        Ok(vec![format!(
            "CVR {tr_id} 1.0.0000 1.0.0000 1.0.0000 https://r2m.camposs.net/storage https://r2m.camposs.net\r\n"
        )])
    }
}
