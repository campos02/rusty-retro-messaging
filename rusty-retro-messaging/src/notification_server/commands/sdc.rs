use super::traits::command::Command;
use crate::error_command::ErrorCommand;

pub struct Sdc;

impl Command for Sdc {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
    ) -> Result<Vec<String>, ErrorCommand> {
        let _ = protocol_version;

        let args: Vec<&str> = command.trim().split(' ').collect();
        let tr_id = args[1];

        Ok(vec![format!("SDC {tr_id} OK\r\n")])
    }
}
