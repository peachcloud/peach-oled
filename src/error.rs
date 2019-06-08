extern crate linux_embedded_hal as hal;

use jsonrpc_core::{types::error::Error, ErrorCode};
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum OledError {
    #[snafu(display("Coordinate {} out of range {}: {}", coord, range, value))]
    InvalidCoordinate {
        coord: String,
        range: String,
        value: i32,
    },

    #[snafu(display("String length out of range 0-21: {}", len))]
    InvalidString { len: usize },

    #[snafu(display("Font size invalid: {}", font))]
    InvalidFontSize { font: String },

    #[snafu(display("Missing expected parameter: {}", e))]
    MissingParameter { e: Error },

    #[snafu(display("Failed to parse parameter: {}", e))]
    ParseError { e: Error },

    #[snafu(display("Failed to create interface for I2C device: {}", source))]
    I2CError {
        source: hal::i2cdev::linux::LinuxI2CError,
    },
}

impl From<OledError> for Error {
    fn from(err: OledError) -> Self {
        match &err {
            OledError::InvalidString { len } => Error {
                code: ErrorCode::ServerError(1),
                message: format!("Validation error: string length {} out of range 0-21.", len),
                data: None,
            },
            OledError::InvalidCoordinate {
                coord,
                value,
                range,
            } => Error {
                code: ErrorCode::ServerError(1),
                message: format!(
                    "Validation error: coordinate {} out of range {}: {}.",
                    coord, range, value
                ),
                data: None,
            },
            OledError::InvalidFontSize { font } => Error {
                code: ErrorCode::ServerError(1),
                message: format!("Validation error: {} is not an accepted font size.", font),
                data: None,
            },
            OledError::I2CError { source } => Error {
                code: ErrorCode::ServerError(2),
                message: "I2C device error.".to_string(),
                data: Some(format!("{}", source).into()),
            },
            OledError::MissingParameter { e } => e.clone(),
            OledError::ParseError { e } => e.clone(),
        }
    }
}
