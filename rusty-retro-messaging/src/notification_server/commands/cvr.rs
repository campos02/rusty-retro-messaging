use super::traits::command::Command;
use crate::error_command::ErrorCommand;

pub struct Cvr;

impl Command for Cvr {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = *args.get(1).ok_or(ErrorCommand::Command("".to_string()))?;

        Ok(vec![format!(
            "CVR {tr_id} 1.0.0000 1.0.0000 7.0.0425 http://download.microsoft.com/download/D/F/B/DFB59A5D-92DF-4405-9767-43E3DF10D25B/fr/Install_MSN_Messenger.exe http://messenger.msn.com/fr\r\n"
        )])
    }
}
