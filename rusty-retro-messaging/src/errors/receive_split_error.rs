use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReceiveSplitError {
    #[error("Client disconnected")]
    Disconnected,
    #[error("Client sent invalid length")]
    InvalidLength,
}
