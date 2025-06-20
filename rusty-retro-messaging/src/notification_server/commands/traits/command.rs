use crate::error_command::ErrorCommand;

pub trait Command {
    fn handle(&self, protocol_version: usize, command: &str) -> Result<Vec<String>, ErrorCommand>;
}
