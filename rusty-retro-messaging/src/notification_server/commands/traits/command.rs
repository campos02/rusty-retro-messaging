use crate::error_command::ErrorCommand;

pub trait Command {
    fn handle(
        &self,
        protocol_version: usize,
        command: &String,
    ) -> Result<Vec<String>, ErrorCommand>;
}
