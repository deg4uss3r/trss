use confy::ConfyError;

use std::error;
use std::fmt;

pub enum Error {
    ConfigurationError(ConfyError),
    WriteError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigurationError(confy_error) => write!(
                f,
                "An error occured in the configuration framework: {}",
                confy_error
            ),
            Error::WriteError(io_error) => write!(
                f,
                "An error occurred when writing to a file or the terminal: {}",
                io_error
            ),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigurationError(confy_error) => write!(
                f,
                "An error occured in the configuration framework: {}",
                confy_error
            ),
            Error::WriteError(io_error) => write!(
                f,
                "An error occurred when writing to a file or the terminal: {}",
                io_error
            ),
        }
    }
}

impl From<ConfyError> for Error {
    fn from(value: ConfyError) -> Self {
        Self::ConfigurationError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::WriteError(value)
    }
}

impl error::Error for Error {}
