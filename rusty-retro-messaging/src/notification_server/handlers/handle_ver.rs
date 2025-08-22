use crate::errors::command_error::CommandError;
use crate::notification_server::commands::ver::Ver;
use crate::notification_server::handlers::process_command::process_command;
use log::{trace, warn};
use tokio::net::tcp::WriteHalf;

pub async fn handle_ver(
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let command = str::from_utf8(&command)?;
    let args: Vec<&str> = command.trim().split(' ').collect();
    trace!("C: {command}");

    match *args.first().unwrap_or(&"") {
        "VER" => {
            let responses = process_command(0, wr, &Ver, command).await?;
            let reply = responses
                .first()
                .ok_or(CommandError::CouldNotGetProtocolVersion)?;

            let args: Vec<&str> = reply.trim().split(' ').collect();
            if *args.first().unwrap_or(&"") == "VER" {
                return Ok(args
                    .get(2)
                    .unwrap_or(&"")
                    .replace("MSNP", "")
                    .parse::<usize>()?);
            }
        }

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Err(CommandError::CouldNotGetProtocolVersion.into())
}
