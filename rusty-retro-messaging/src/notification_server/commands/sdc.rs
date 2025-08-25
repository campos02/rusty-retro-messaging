use super::traits::command::Command;
use crate::errors::command_error::CommandError;

pub struct Sdc;

impl Command for Sdc {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;

        Ok(vec![format!("SDC {tr_id} OK\r\n")])
    }
}
