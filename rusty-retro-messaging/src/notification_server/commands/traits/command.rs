use crate::errors::command_error::CommandError;

pub trait Command {
    async fn handle(
        &self,
        protocol_version: usize,
        command: &str,
    ) -> Result<Vec<String>, CommandError>;
}
