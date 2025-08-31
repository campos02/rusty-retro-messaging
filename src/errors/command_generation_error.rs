use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandGenerationError {
    #[error("User has no presence")]
    NoPresence,
    #[error("User has no client ID")]
    NoClientId,
    #[error("Original command has no transaction ID")]
    NoTrId,
    #[error("The Switchboard IP environment variable is not set")]
    SwitchboardIpNotSet,
    #[error("Could not get personal message")]
    CouldNotGetPersonalMessage,
}
