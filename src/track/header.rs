use super::{error::TrackFileError, fixedstr::FixedStr};
use chrono::Utc;

const HEADER_MAGIC_NUMBER: u64 = 0xfb9cfc9b116a158e;
const HEADER_VERSION: u64 = 1;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Header {
  magic: u64,
  version: u64,
  updated_at: u64,
  count: u64,
  flight_id: FixedStr<36>,
  departure: FixedStr<8>,
  arrival: FixedStr<8>,
}

impl Header {
  pub fn new(flight_id: &str) -> Result<Self, TrackFileError> {
    Ok(Self {
      magic: HEADER_MAGIC_NUMBER,
      version: HEADER_VERSION,
      updated_at: Utc::now().timestamp_millis() as u64,
      count: 0,
      flight_id: flight_id.into(),
      departure: FixedStr::default(),
      arrival: FixedStr::default(),
    })
  }

  pub fn check_magic(&self) -> bool {
    self.magic == HEADER_MAGIC_NUMBER
  }

  pub fn version(&self) -> u64 {
    self.version
  }

  pub fn timestamp(&self) -> u64 {
    self.updated_at
  }

  pub fn count(&self) -> u64 {
    self.count
  }

  pub fn touch(&mut self) {
    self.updated_at = Utc::now().timestamp_millis() as u64;
  }

  pub fn inc(&mut self) {
    self.count += 1;
    self.touch();
  }

  pub fn set_departure(&mut self, departure: &str) {
    self.departure.set(departure);
    self.touch();
  }

  pub fn set_arrival(&mut self, arrival: &str) {
    self.arrival.set(arrival);
    self.touch();
  }

  pub fn get_departure(&self) -> String {
    let dep = &self.departure;
    dep.into()
  }

  pub fn get_arrival(&self) -> String {
    let arr = &self.arrival;
    arr.into()
  }

  pub fn get_flight_id(&self) -> String {
    let fid = &self.flight_id;
    fid.into()
  }
}
