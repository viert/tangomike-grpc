use figment::{
  providers::{Format, Toml},
  Error, Figment,
};
use log::LevelFilter;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TrackConfig {
  pub folder: String,
}

const DEFAULT_LEVEL: fn() -> LevelFilter = || LevelFilter::Debug;
const DEFAULT_BIND: fn() -> String = || "127.0.0.1:9100".to_owned();

#[derive(Debug, Deserialize)]
pub struct LogConfig {
  #[serde(default = "DEFAULT_LEVEL")]
  pub level: LevelFilter,
}

impl Default for LogConfig {
  fn default() -> Self {
    Self {
      level: DEFAULT_LEVEL(),
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
  #[serde(default = "DEFAULT_BIND")]
  pub bind: String,
}

impl Default for ServiceConfig {
  fn default() -> Self {
    Self {
      bind: DEFAULT_BIND(),
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub track: TrackConfig,
  #[serde(default)]
  pub log: LogConfig,
  #[serde(default)]
  pub service: ServiceConfig,
}

pub fn read_in_config<P: AsRef<Path>>(filename: P) -> Result<Config, Error> {
  Figment::new().merge(Toml::file(filename)).extract()
}
