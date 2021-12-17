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

    #[error("Protocol error: {0}")]
    ProtocolError(&'static str),
}

pub type SoapboxResult<T> = Result<T, SoapboxError>;
