use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Could not get protocol version")]
    CouldNotGetProtocolVersion,
    #[error("Could not get authenticated user")]
    CouldNotGetAuthenticatedUser,
    #[error("Could not get session")]
    CouldNotGetSession,
    #[error("Could not get session receiver")]
    CouldNotGetSessionReceiver,
    #[error("Could not get principals from session, lock poisoned")]
    PrincipalsLockError,
    #[error("Client disconnected")]
    Disconnected,
}
