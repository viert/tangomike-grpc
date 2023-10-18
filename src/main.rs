use clap::Parser;
use log::{error, info};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tangomike_grpc::{
  config::read_in_config,
  geodata::GeoData,
  service::{tangomike::track_server::TrackServer, TrackService},
  track::store::TrackStore,
};
use tonic::transport::Server;

#[derive(Parser, Debug)]
struct Args {
  #[arg(short, default_value = "/etc/tangomike/tangomike.toml")]
  config: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();
  let cfg = read_in_config(&args.config)?;

  TermLogger::init(
    cfg.log.level,
    Config::default(),
    TerminalMode::Stdout,
    ColorChoice::Always,
  )
  .unwrap();

  info!("TangoMikeFoxtrot server version {}", VERSION);
  info!("using log level {}", cfg.log.level);

  let res = GeoData::load().await;
  let geo = match res {
    Err(err) => {
      error!("error loading geodata: {err}");
      return Ok(());
    }
    Ok(geo) => geo,
  };

  let bind = &cfg.service.bind;
  let addr = match bind.parse() {
    Ok(addr) => addr,
    Err(err) => {
      error!("error parsing service.bind: {err}");
      return Ok(());
    }
  };

  let store = TrackStore::new(&cfg.track);
  let svc = TrackService::new(geo, store);
  let svc = TrackServer::new(svc);

  info!("starting grpc service...");
  Server::builder().add_service(svc).serve(addr).await?;

  // let ts = TrackStore::new(&cfg.track);
  // let mut tf = ts.get_track_file("3fa6de2e-94e7-4d4b-b46c-76f222538b31")?;
  // let entry = TrackFileEntry::TrackPoint(TrackPoint {
  //   ts: Utc::now().timestamp_millis() as u64,
  //   lat: 51.148102,
  //   lng: -0.190278,
  //   hdg_true: 0.0,
  //   alt_amsl: 3540.0,
  //   alt_agl: 3221.0,
  //   gnd_height: 319.0,
  //   crs: 1.0,
  //   ias: 242.0,
  //   tas: 280.0,
  //   gs: 312.0,
  //   ap_master: true,
  //   gear_pct: 0,
  //   flaps: 0,
  //   on_gnd: false,
  //   on_rwy: false,
  //   wind_vel: 18.0,
  //   wind_dir: 180.0,
  //   distance: 0.0,
  // });
  // tf.append(&entry)?;

  // let header = tf.get_header();
  // println!("{header:?}");

  // let gd = GeoData::load().await?;
  // let airport = gd.closest(-0.190273, 51.148100);
  // println!("{airport:?}");
  Ok(())
}
