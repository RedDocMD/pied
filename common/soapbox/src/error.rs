use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoapboxError {
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),

    #[error("Serial connection error: {0}")]
    SerialError(#[from] serialport::Error),

    #[error("Timeout error: {0}s")]
    TimeoutError(u64),
}

pub type SoapboxResult<T> = Result<T, SoapboxError>;
