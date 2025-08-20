use crate::notification_server::handlers::process_command::process_command;
use crate::{error_command::ErrorCommand, notification_server::commands::ver::Ver};
use log::{trace, warn};
use tokio::net::tcp::WriteHalf;

pub async fn handle_ver(
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<Option<usize>, ErrorCommand> {
    let command = str::from_utf8(&command).or(Err(ErrorCommand::Disconnect(
        "Command contained invalid UTF-8".to_string(),
    )))?;

    let args: Vec<&str> = command.trim().split(' ').collect();
    trace!("C: {command}");

    match *args.first().unwrap_or(&"") {
        "VER" => {
            let responses = process_command(0, wr, &Ver, command).await?;
            let reply = responses
                .first()
                .ok_or(ErrorCommand::Disconnect("".to_string()))?;

            let args: Vec<&str> = reply.trim().split(' ').collect();
            if *args.first().unwrap_or(&"") == "VER" {
                return Ok(Some(
                    args.get(2)
                        .unwrap_or(&"")
                        .replace("MSNP", "")
                        .parse::<usize>()
                        .or(Err(ErrorCommand::Disconnect("".to_string())))?,
                ));
            }
        }

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Ok(None)
}
