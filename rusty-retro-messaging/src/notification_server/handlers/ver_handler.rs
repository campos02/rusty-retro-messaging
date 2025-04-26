use super::command_handler::CommandHandler;
use super::command_processor::CommandProcessor;
use crate::{error_command::ErrorCommand, notification_server::commands::ver::Ver};
use log::warn;
use tokio::net::tcp::WriteHalf;

pub struct VerHandler {
    pub protocol_version: Option<usize>,
}

impl VerHandler {
    pub fn new() -> Self {
        VerHandler {
            protocol_version: None,
        }
    }
}

impl CommandHandler for VerHandler {
    async fn handle_command(
        &mut self,
        sender: String,
        wr: &mut WriteHalf<'_>,
        command: String,
    ) -> Result<(), ErrorCommand> {
        let _ = sender;
        let args: Vec<&str> = command.trim().split(' ').collect();

        match args[0] {
            "VER" => {
                let responses = Self::process_command(0, wr, &mut Ver, &command).await?;
                let reply = &responses[0];

                let args: Vec<&str> = reply.trim().split(' ').collect();
                if args[0] == "VER" {
                    self.protocol_version = Some(
                        args[2]
                            .replace("MSNP", "")
                            .parse::<usize>()
                            .expect("Could not get protocol version"),
                    );
                }
            }

            _ => warn!("Unmatched command before authentication: {command}"),
        }

        Ok(())
    }
}
