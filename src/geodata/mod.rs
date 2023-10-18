use chrono::Utc;
use log::info;
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use serde::Deserialize;
use std::error::Error;

use crate::util::seconds_since;

const OURAIRPORTS_URL: &str =
  "https://raw.githubusercontent.com/viert/ourairports-json/main/output/airport_list.json";

#[derive(Debug, Deserialize)]
pub struct Airport {
  pub id: u32,
  pub ident: String,
  #[serde(rename = "type")]
  pub airport_type: String,
  pub name: String,
  #[serde(rename = "latitude_deg")]
  pub lat: f64,
  #[serde(rename = "longitude_deg")]
  pub lng: f64,
  pub elevation_ft: Option<i32>,
  pub gps_code: String,
  pub iata_code: String,
  pub local_code: String,
  pub home_link: String,
  pub wikipedia_link: String,
  pub keywords: Vec<String>,
}

impl RTreeObject for Airport {
  type Envelope = AABB<(f64, f64)>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_point((self.lng, self.lat))
  }
}

impl PointDistance for Airport {
  fn distance_2(
    &self,
    point: &<Self::Envelope as rstar::Envelope>::Point,
  ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
    let (lng, lat) = *point;
    let lat_diff = (lat - self.lat).abs();
    let mut lng_diff = lng - self.lng;
    if lng_diff < -180.0 {
      lng_diff += 360.0;
    } else if lng_diff < 0.0 {
      lng_diff = lng_diff.abs();
    }
    lat_diff * lat_diff + lng_diff * lng_diff
  }
}

#[derive(Debug)]
pub struct GeoData {
  airports: RTree<Airport>,
}

impl GeoData {
  pub fn new(airports: Vec<Airport>) -> Self {
    info!("indexing geodata...");
    let t1 = Utc::now();
    let mut tree = RTree::new();
    for airport in airports {
      tree.insert(airport);
    }
    info!("geodata indexed in {}s", seconds_since(t1));
    Self { airports: tree }
  }

  pub async fn load() -> Result<Self, Box<dyn Error>> {
    info!("loading geodata...");
    let t1 = Utc::now();
    let raw = reqwest::get(OURAIRPORTS_URL).await?.text().await?;
    info!("geodata loaded in {}s", seconds_since(t1));
    info!("parsing geodata...");
    let t1 = Utc::now();
    let airports: Vec<Airport> = serde_json::from_str(&raw)?;
    info!("geodata parsed in {}s", seconds_since(t1));
    Ok(Self::new(airports))
  }

  pub fn closest_airport(&self, lng: f64, lat: f64) -> Option<&Airport> {
    self.airports.nearest_neighbor(&(lng, lat))
  }
}
