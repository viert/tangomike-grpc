use std::{error::Error, fmt::Display};

use tonic::Status;

#[derive(Debug)]
pub enum TrackFileError {
  IOError(std::io::Error),
  InvalidMagicNumber,
  InvalidFileLength(usize, usize),
  InsufficientDataLength(String, usize),
  IndexError(usize),
  NotFound(String),
  InvalidFlightId(&'static str),
}

impl Display for TrackFileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TrackFileError::IOError(err) => write!(f, "TrackFileError: {err}"),
      TrackFileError::InvalidMagicNumber => write!(f, "Track file corrupted, invalid magic number"),
      TrackFileError::InvalidFileLength(expected, got) => write!(
        f,
        "Invalid track file length: expected {expected}, got {got}"
      ),
      TrackFileError::InsufficientDataLength(ident, size) => {
        write!(f, "Insufficient data length while parsing {ident}: {size}")
      }
      TrackFileError::IndexError(idx) => {
        write!(f, "Invalid index {idx} while reading track file data")
      }
      TrackFileError::NotFound(filename) => {
        write!(f, "Track file {filename} not found")
      }
      TrackFileError::InvalidFlightId(err) => {
        write!(f, "FlightId is incorrect: {err}")
      }
    }
  }
}

impl Error for TrackFileError {}

impl From<std::io::Error> for TrackFileError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}

impl From<TrackFileError> for Status {
  fn from(value: TrackFileError) -> Self {
    match &value {
      TrackFileError::NotFound(err) => Status::not_found(err.to_string()),
      _ => Status::internal(value.to_string()),
    }
  }
}
