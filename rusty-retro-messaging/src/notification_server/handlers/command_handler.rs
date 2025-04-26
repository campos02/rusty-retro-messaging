use super::command_processor::CommandProcessor;
use crate::error_command::ErrorCommand;
use tokio::net::tcp::WriteHalf;

pub trait CommandHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand>;
}

impl<T: CommandHandler> CommandProcessor for T {}
