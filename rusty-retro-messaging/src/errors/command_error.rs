use crate::errors::command_generation_error::CommandGenerationError;
use crate::message::Message;
use thiserror::Error;
use tokio::sync::broadcast::error::{RecvError, SendError};

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Could not get protocol version")]
    CouldNotGetProtocolVersion,
    #[error("Could not get authenticated user")]
    CouldNotGetAuthenticatedUser,
    #[error("Sending error message to client")]
    Reply(String),
    #[error("Sending error message to client and disconnecting")]
    ReplyAndDisconnect(String),
    #[error("Could not get transaction ID from command")]
    NoTrId,
    #[error("Command doesn't have enough arguments")]
    NotEnoughArguments,
    #[error("Could not create NLN reply: {0}")]
    CouldNotCreateNln(CommandGenerationError),
    #[error("Could not create UBX reply: {0}")]
    CouldNotCreateUbx(CommandGenerationError),
    #[error("Could not create RNG reply: {0}")]
    CouldNotCreateRng(CommandGenerationError),
    #[error("Could not send to broadcast: {0}")]
    CouldNotSendToBroadcast(SendError<Message>),
    #[error("Could not receive from broadcast: {0}")]
    CouldNotReceiveFromBroadcast(RecvError),
    #[error("Could not get command from client message")]
    CouldNotGetCommand,
}
