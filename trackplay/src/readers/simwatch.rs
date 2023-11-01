use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use std::{error::Error, fs::File, time::Duration};
use tm_grpc::service::tangomike::{track_message, TrackMessage, TrackPoint};
use tokio::{
  sync::mpsc::{self, Receiver},
  time::sleep,
};

use super::TrackReader;

#[derive(Deserialize, Clone)]
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
      ts: value.ts,
      union: Some(track_message::Union::Point(TrackPoint {
        lat: value.lat,
        lng: value.lng,
        hdg_true: value.hdg as f64,
        alt_amsl: value.alt as f64,
        alt_agl: value.alt as f64,
        gnd_height: 0.0,
        crs: value.hdg as f64,
        ias: value.gs as f64,
        tas: value.gs as f64,
        gs: value.gs as f64,
        ap_master: false,
        gear_pct: 0,
        flaps: 0,
        on_gnd: false,
        on_rwy: false,
        wind_vel: 0.0,
        wind_dir: 0.0,
      })),
    }
  }
}

#[derive(Deserialize)]
pub struct SimwatchPilot {
  pub callsign: String,
  pub track: Vec<SimwatchTrackPoint>,
}

pub struct SimwatchReader {
  path: String,
  data: Option<SimwatchPilot>,
}

impl SimwatchReader {
  pub fn new(path: &str) -> Self {
    Self {
      path: path.to_owned(),
      data: None,
    }
  }

  pub async fn prepare(&mut self) -> Result<(), Box<dyn Error>> {
    let is_local_file = !(self.path.starts_with("http://") || self.path.starts_with("https://"));
    let data: SimwatchPilot = if is_local_file {
      let f = File::open(&self.path)?;
      serde_json::from_reader(f)?
    } else {
      reqwest::get(&self.path).await?.json().await?
    };
    self.data = Some(data);
    Ok(())
  }

  pub fn atc_id(&self) -> Option<String> {
    self.data.as_ref().map(|d| d.callsign.to_owned())
  }
}

#[async_trait]
impl TrackReader for SimwatchReader {
  async fn read(&self) -> Result<Receiver<TrackMessage>, Box<dyn Error>> {
    let (tx, rx) = mpsc::channel(1024);
    let points: Vec<SimwatchTrackPoint> =
      self.data.as_ref().unwrap().track.iter().cloned().collect();
    if points.len() > 0 {
      let time_start = points[0].ts as i64;
      let now = Utc::now().timestamp_millis() as i64;
      let timediff = now - time_start;
      println!(
        "Time passed since the track was recorded {:?}",
        Duration::from_millis(timediff as u64)
      );

      tokio::spawn(async move {
        for point in points {
          let ts = point.ts as i64;
          let adj_now = Utc::now().timestamp_millis() as i64 - timediff;

          if ts > adj_now {
            let sleep_time = (ts - adj_now) as i64;
            if sleep_time > 0 {
              let sleep_time = if sleep_time > 10000 {
                10000
              } else {
                sleep_time
              };
              let sleep_time = Duration::from_millis(sleep_time as u64);
              println!("sleeping for {sleep_time:?}");
              sleep(sleep_time).await;
            }
          }

          let mut msg: TrackMessage = point.into();
          msg.ts = Utc::now().timestamp_millis() as u64;
          let res = tx.send(msg).await;
          if let Err(err) = res {
            println!("Error sending entry: {err}");
          }
        }
      });
    }

    Ok(rx)
  }
}
