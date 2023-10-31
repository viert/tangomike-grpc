use serde::Deserialize;
use tm_grpc::service::tangomike::{track_message, TrackMessage, TrackPoint};

#[derive(Deserialize)]
pub struct SimwatchTrackPoint {
  pub lat: f64,
  pub lng: f64,
  pub alt: i32,
  pub hdg: i32,
  pub gs: i32,
  pub ts: u64,
}

impl From<SimwatchTrackPoint> for TrackMessage {
  fn from(value: SimwatchTrackPoint) -> Self {
    Self {
      union: Some(track_message::Union::Point(TrackPoint {
        ts: todo!(),
        lat: todo!(),
        lng: todo!(),
        hdg_true: todo!(),
        alt_amsl: todo!(),
        alt_agl: todo!(),
        gnd_height: todo!(),
        crs: todo!(),
        ias: todo!(),
        tas: todo!(),
        gs: todo!(),
        ap_master: todo!(),
        gear_pct: todo!(),
        flaps: todo!(),
        on_gnd: todo!(),
        on_rwy: todo!(),
        wind_vel: todo!(),
        wind_dir: todo!(),
      })),
    }
  }
}

#[derive(Deserialize)]
pub struct SimwatchPilot {
  pub callsign: String,
  pub track: Vec<SimwatchTrackPoint>,
}
