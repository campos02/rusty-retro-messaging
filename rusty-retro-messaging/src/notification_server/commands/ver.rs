use super::traits::command::Command;
use crate::errors::command_error::CommandError;

pub struct Ver;

impl Command for Ver {
    async fn handle(
        &self,
        protocol_version: u32,
        command: &str,
    ) -> Result<Vec<String>, CommandError> {
        let _ = protocol_version;

        let versions = ["MSNP12", "MSNP11", "MSNP10", "MSNP9", "MSNP8"];
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(CommandError::NoTrId)?;

        for arg in &args {
            for version in &versions {
                if *arg == *version {
                    return Ok(vec![format!("VER {tr_id} {version}\r\n")]);
                }
            }
        }

        Err(CommandError::ReplyAndDisconnect(format!(
            "VER {tr_id} 0\r\n"
        )))
    }
}
