use inquire::error::InquireError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeekoCliError {
    #[error("Input error: {0}")]
    Input(String),
    #[error("{0}")]
    ReaderRuntime(#[from] peeko::reader::ImageReaderError),
    #[error("{0}")]
    RegistryRuntime(#[from] peeko::registry::RegistryError),
    #[error("{0}")]
    RuntimeError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    InteractionError(#[from] InquireError),
}

pub type Result<T> = std::result::Result<T, PeekoCliError>;
