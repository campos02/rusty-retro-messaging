use super::traits::command::Command;
use crate::error_command::ErrorCommand;

pub struct Ver;

impl Command for Ver {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let versions = vec!["MSNP12", "MSNP11"];
        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;

        for i in &args {
            for version in &versions {
                if *i == *version {
                    return Ok(vec![format!("VER {tr_id} {version}\r\n")]);
                }
            }
        }

        Err(ErrorCommand::Disconnect(format!("VER {tr_id} 0\r\n")))
    }
}
