use clap::Parser;
use log::{error, info};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tm_grpc::{
  config::read_in_config,
  geodata::GeoData,
  service::{tangomike::track_server::TrackServer, TrackService},
  track::store::TrackStore,
};
use tonic::transport::Server;

#[derive(Parser, Debug)]
struct Args {
  #[arg(short, default_value = "/etc/tangomike/tm-grpc.toml")]
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
  let svc = TrackService::new(geo, store, &cfg.api.base_uri);
  let svc = TrackServer::new(svc);

  info!("starting grpc service...");
  Server::builder().add_service(svc).serve(addr).await?;

  Ok(())
}
