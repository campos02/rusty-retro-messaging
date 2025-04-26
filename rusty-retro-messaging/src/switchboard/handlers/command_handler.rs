use super::command_processor::CommandProcessor;
use crate::error_command::ErrorCommand;
use tokio::net::tcp::WriteHalf;

pub trait CommandHandler {
    async fn handle_command(
        &mut self,
        wr: &mut WriteHalf<'_>,
        base64_command: String,
    ) -> Result<(), ErrorCommand>;
}

impl<T: CommandHandler> CommandProcessor for T {}
