use crate::notification_server::handlers::process_command::process_command;
use crate::{error_command::ErrorCommand, notification_server::commands::ver::Ver};
use log::warn;
use tokio::net::tcp::WriteHalf;

pub async fn handle_ver(
    wr: &mut WriteHalf<'_>,
    command: Vec<u8>,
) -> Result<Option<usize>, ErrorCommand> {
    let command = str::from_utf8(&command).expect("Command contained invalid UTF-8");
    let args: Vec<&str> = command.trim().split(' ').collect();

    match args[0] {
        "VER" => {
            let responses = process_command(0, wr, &Ver, command).await?;
            let reply = &responses[0];

            let args: Vec<&str> = reply.trim().split(' ').collect();
            if args[0] == "VER" {
                return Ok(Some(
                    args[2]
                        .replace("MSNP", "")
                        .parse::<usize>()
                        .expect("Could not get protocol version"),
                ));
            }
        }

        _ => warn!("Unmatched command before authentication: {command}"),
    }

    Ok(None)
}
