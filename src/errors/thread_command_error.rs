use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadCommandError {
    #[error("Could not get receiver")]
    ReceivingError,
    #[error("Could not get protocol version")]
    CouldNotGetProtocolVersion,
    #[error("Could not get authenticated user")]
    CouldNotGetAuthenticatedUser,
    #[error("User logged in on another computer")]
    UserLoggedInOnAnotherComputer,
    #[error("Command doesn't have enough arguments: {0}")]
    NotEnoughArguments(String),
}
