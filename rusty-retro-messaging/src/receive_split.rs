use crate::errors::receive_split_error::ReceiveSplitError;
use core::str;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::ReadHalf;

pub async fn receive_split(rd: &mut ReadHalf<'_>) -> Result<Vec<Vec<u8>>, ReceiveSplitError> {
    let mut buf = vec![0; 1664];
    let received = rd.read(&mut buf).await.unwrap_or(0);

    if received == 0 {
        return Err(ReceiveSplitError::Disconnected);
    }

    let mut messages_bytes = buf.get(..received).unwrap_or_default().to_vec();
    let mut messages = Vec::new();

    loop {
        let messages_string = unsafe { str::from_utf8_unchecked(&messages_bytes) };
        let Some(command) = messages_string.lines().next() else {
            break;
        };

        let command = command.to_string() + "\r\n";
        let args: Vec<&str> = command.trim().split(' ').collect();

        match *args.first().unwrap_or(&"") {
            "UUX" | "MSG" => {
                let length_index = match *args.first().unwrap_or(&"") {
                    "UUX" => 2,
                    _ => 3,
                };

                let length = args
                    .get(length_index)
                    .unwrap_or(&"")
                    .parse::<usize>()
                    .or(Err(ReceiveSplitError::InvalidLength))?;

                let length = command.len() + length;
                if length > messages_bytes.len() {
                    let mut buf = vec![0; 1664];
                    let received = rd.read(&mut buf).await.unwrap_or(0);

                    if received == 0 {
                        return Err(ReceiveSplitError::Disconnected);
                    }

                    let buf = buf.get(..received).unwrap_or_default();
                    messages_bytes.extend_from_slice(buf);
                    continue;
                }

                let new_bytes = messages_bytes.drain(..length);
                messages.push(new_bytes.collect());
            }

            _ => {
                let length = command.len();
                if length > messages_bytes.len() {
                    return Err(ReceiveSplitError::InvalidLength);
                }

                let new_bytes = messages_bytes.drain(..length);
                messages.push(new_bytes.collect());
            }
        }
    }

    Ok(messages)
}
