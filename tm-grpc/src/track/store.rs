use std::{fs, path::PathBuf};

use crate::config::TrackConfig;

use super::{error::TrackFileError, trackfile::TrackFile};

const SUBKEY_LENGTH: usize = 3;
const NESTING_LEVEL: usize = 2;

#[derive(Debug)]
pub struct TrackStore {
  folder: String,
}

impl TrackStore {
  pub fn new(cfg: &TrackConfig) -> Self {
    Self {
      folder: cfg.folder.to_owned(),
    }
  }

  fn check_flight_id(&self, flight_id: &str) -> Result<(), TrackFileError> {
    if flight_id.len() < NESTING_LEVEL * SUBKEY_LENGTH {
      Err(TrackFileError::InvalidFlightId("ID is too short or empty"))
    } else {
      Ok(())
    }
  }

  fn target_dir(&self, flight_id: &str) -> PathBuf {
    let mut path = PathBuf::from(&self.folder);
    for i in 0..NESTING_LEVEL {
      let subkey = &flight_id[i * SUBKEY_LENGTH..(i + 1) * SUBKEY_LENGTH];
      path = path.join(subkey);
    }
    path
  }

  pub fn open_or_create(&self, flight_id: &str) -> Result<TrackFile, TrackFileError> {
    self.check_flight_id(flight_id)?;
    let path = self.target_dir(flight_id);
    fs::create_dir_all(&path)?;
    let path = path.join(format!("{flight_id}.bin"));
    TrackFile::new(path, flight_id)
  }

  pub fn open(&self, flight_id: &str) -> Result<TrackFile, TrackFileError> {
    self.check_flight_id(flight_id)?;
    let path = self.target_dir(flight_id);
    let path = path.join(format!("{flight_id}.bin"));
    TrackFile::open(path)
  }
}
