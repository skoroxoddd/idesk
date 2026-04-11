use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Capture error: {0}")]
    Capture(String),

    #[error("Encoding error: {0}")]
    Encode(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Signaling error: {0}")]
    Signaling(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
