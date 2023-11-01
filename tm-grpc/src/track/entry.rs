use crate::service::tangomike::{self, track_message::Union, TrackMessage};

impl From<&TrackPoint> for haversine::Location {
  fn from(value: &TrackPoint) -> Self {
    Self {
      latitude: value.lat,
      longitude: value.lng,
    }
  }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct TrackPoint {
  pub ts: u64,
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

impl From<TrackPoint> for tangomike::TrackPoint {
  fn from(value: TrackPoint) -> Self {
    Self {
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
    }
  }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TouchDown {
  pub ts: u64,
  pub bank: f64,
  pub hdg_mag: f64,
  pub hdg_true: f64,
  pub vel_nrm: f64,
  pub pitch: f64,
  pub lat: f64,
  pub lng: f64,
}

impl From<TouchDown> for tangomike::TouchDown {
  fn from(value: TouchDown) -> Self {
    Self {
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

#[derive(Debug, Clone)]
#[repr(C)]
pub enum TrackFileEntry {
  TrackPoint(TrackPoint),
  TouchDown(TouchDown),
}

impl From<TrackFileEntry> for TrackMessage {
  fn from(value: TrackFileEntry) -> Self {
    match value {
      TrackFileEntry::TrackPoint(tp) => Self {
        ts: tp.ts,
        union: Some(Union::Point(tangomike::TrackPoint {
          lat: tp.lat,
          lng: tp.lng,
          hdg_true: tp.hdg_true,
          alt_amsl: tp.alt_amsl,
          alt_agl: tp.alt_agl,
          gnd_height: tp.gnd_height,
          crs: tp.crs,
          ias: tp.ias,
          tas: tp.tas,
          gs: tp.gs,
          ap_master: tp.ap_master,
          gear_pct: tp.gear_pct,
          flaps: tp.flaps,
          on_gnd: tp.on_gnd,
          on_rwy: tp.on_rwy,
          wind_vel: tp.wind_vel,
          wind_dir: tp.wind_dir,
        })),
      },
      TrackFileEntry::TouchDown(td) => Self {
        ts: td.ts,
        union: Some(Union::TouchDown(tangomike::TouchDown {
          bank: td.bank,
          hdg_mag: td.hdg_mag,
          hdg_true: td.hdg_true,
          vel_nrm: td.vel_nrm,
          pitch: td.pitch,
          lat: td.lat,
          lng: td.lng,
        })),
      },
    }
  }
}

impl From<TrackMessage> for TrackFileEntry {
  fn from(value: TrackMessage) -> Self {
    match value.union.unwrap() {
      Union::Point(point) => Self::TrackPoint(TrackPoint {
        ts: value.ts,
        lat: point.lat,
        lng: point.lng,
        hdg_true: point.hdg_true,
        alt_amsl: point.alt_amsl,
        alt_agl: point.alt_agl,
        gnd_height: point.gnd_height,
        crs: point.crs,
        ias: point.ias,
        tas: point.tas,
        gs: point.gs,
        ap_master: point.ap_master,
        gear_pct: point.gear_pct,
        flaps: point.flaps,
        on_gnd: point.on_gnd,
        on_rwy: point.on_rwy,
        wind_vel: point.wind_vel,
        wind_dir: point.wind_dir,
        distance: 0.0,
      }),
      Union::TouchDown(td) => Self::TouchDown(TouchDown {
        ts: value.ts,
        bank: td.bank,
        hdg_mag: td.hdg_mag,
        hdg_true: td.hdg_true,
        vel_nrm: td.vel_nrm,
        pitch: td.pitch,
        lat: td.lat,
        lng: td.lng,
      }),
    }
  }
}

impl PartialEq for TrackFileEntry {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::TrackPoint(l0), Self::TrackPoint(r0)) => {
        l0.lat == r0.lat
          && l0.lng == r0.lng
          && l0.hdg_true == r0.hdg_true
          && l0.alt_amsl == r0.alt_amsl
          && l0.alt_agl == r0.alt_agl
          && l0.gnd_height == r0.gnd_height
          && l0.crs == r0.crs
          && l0.ias == r0.ias
          && l0.tas == r0.tas
          && l0.gs == r0.gs
          && l0.ap_master == r0.ap_master
          && l0.gear_pct == r0.gear_pct
          && l0.flaps == r0.flaps
          && l0.on_gnd == r0.on_gnd
          && l0.on_rwy == r0.on_rwy
          && l0.wind_dir == r0.wind_dir
          && l0.wind_vel == r0.wind_vel
          && l0.distance == r0.distance
      }
      (Self::TouchDown(_), Self::TouchDown(_)) => false,
      _ => false,
    }
  }
}
