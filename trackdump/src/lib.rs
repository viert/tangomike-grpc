use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Serialize;
use tm_grpc::track::{
  entry::{self, TrackFileEntry},
  header::Header,
  trackfile::TrackFile,
};

#[derive(Parser, Debug)]
pub struct Args {
  filename: String,

  /// include points
  #[arg(short)]
  points: bool,

  #[arg(long)]
  json: bool,
}

#[derive(Debug, Serialize)]
struct TrackPoint {
  pub ts: DateTime<Utc>,
  pub lat: f64,
  pub lng: f64,
  pub hdg_true: f64,
  pub alt_amsl: f64,
  pub alt_agl: f64,
  pub gnd_height: f64,
  pub crs: f64,
  pub ias: f64,
  pub tas: f64,
  pub gs: f64,
  pub ap_master: bool,
  pub gear_pct: i64,
  pub flaps: i64,
  pub on_gnd: bool,
  pub on_rwy: bool,
  pub wind_vel: f64,
  pub wind_dir: f64,
  pub distance: f64,
}

impl From<entry::TrackPoint> for TrackPoint {
  fn from(value: entry::TrackPoint) -> Self {
    Self {
      ts: datetime_from_timestamp(value.ts),
      lat: value.lat,
      lng: value.lng,
      hdg_true: value.hdg_true,
      alt_amsl: value.alt_amsl,
      alt_agl: value.alt_agl,
      gnd_height: value.gnd_height,
      crs: value.crs,
      ias: value.ias,
      tas: value.tas,
      gs: value.gs,
      ap_master: value.ap_master,
      gear_pct: value.gear_pct,
      flaps: value.flaps,
      on_gnd: value.on_gnd,
      on_rwy: value.on_rwy,
      wind_vel: value.wind_vel,
      wind_dir: value.wind_dir,
      distance: value.distance,
    }
  }
}

#[derive(Debug, Serialize)]
struct TouchDown {
  pub ts: DateTime<Utc>,
  pub bank: f64,
  pub hdg_mag: f64,
  pub hdg_true: f64,
  pub vel_nrm: f64,
  pub pitch: f64,
  pub lat: f64,
  pub lng: f64,
}

impl From<entry::TouchDown> for TouchDown {
  fn from(value: entry::TouchDown) -> Self {
    Self {
      ts: datetime_from_timestamp(value.ts),
      bank: value.bank,
      hdg_mag: value.hdg_mag,
      hdg_true: value.hdg_true,
      vel_nrm: value.vel_nrm,
      pitch: value.pitch,
      lat: value.lat,
      lng: value.lng,
    }
  }
}

#[derive(Debug, Serialize)]
struct TrackDump {
  pub magic: String,
  pub version: u64,
  pub updated_at: DateTime<Utc>,
  pub count: u64,
  pub flight_id: String,
  pub departure: String,
  pub arrival: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub points: Option<Vec<TrackPoint>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub touchdowns: Option<Vec<TouchDown>>,
}

impl From<Header> for TrackDump {
  fn from(value: Header) -> Self {
    Self {
      magic: format!("{:X}", value.magic),
      version: value.version,
      updated_at: datetime_from_timestamp(value.updated_at),
      count: value.count,
      flight_id: value.get_flight_id(),
      departure: value.get_departure(),
      arrival: value.get_arrival(),
      points: None,
      touchdowns: None,
    }
  }
}

fn datetime_from_timestamp(ts: u64) -> DateTime<Utc> {
  let secs = (ts / 1000) as i64;
  let nsecs = (ts % 1000 * 1_000_000) as u32;
  DateTime::from_timestamp(secs, nsecs).unwrap()
}

pub fn dump_trackfile() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();
  let tf = TrackFile::open(&args.filename)?;
  let mut dump: TrackDump = tf.get_header()?.into();

  if args.points {
    let mut points = vec![];
    let mut touchdowns = vec![];
    for entry in tf.read_all()? {
      match entry {
        TrackFileEntry::TrackPoint(tp) => points.push(tp.into()),
        TrackFileEntry::TouchDown(td) => touchdowns.push(td.into()),
      }
    }
    dump.points = Some(points);
    dump.touchdowns = Some(touchdowns);
  }

  let dump = if args.json {
    serde_json::to_string(&dump)?
  } else {
    serde_yaml::to_string(&dump)?
  };

  println!("{dump}");
  Ok(())
}
