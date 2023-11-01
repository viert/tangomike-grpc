use clap::{builder::PossibleValue, Parser, ValueEnum};
use trackplay::{
  readers::{simwatch::SimwatchReader, trackfile::TrackFileReader, TrackReader},
  sender::Sender,
};

#[derive(Debug, Clone)]
enum ServiceType {
  TrackFile,
  Simwatch,
}

impl ValueEnum for ServiceType {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::TrackFile, Self::Simwatch]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    match self {
      ServiceType::TrackFile => Some(PossibleValue::new("trackfile")),
      ServiceType::Simwatch => Some(PossibleValue::new("simwatch")),
    }
  }
}

#[derive(Parser, Debug)]
pub struct Args {
  path: String,

  #[arg(short, long)]
  domain: Option<String>,

  #[arg(short, long)]
  token: String,

  #[arg(long, default_value_t = 9200)]
  grpc_port: u16,

  #[arg(long)]
  atc_id: Option<String>,

  #[arg(short)]
  service_type: ServiceType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();
  let token = args.token;
  let domain = args.domain.unwrap_or("tmf.vatsimnerd.com".into());
  let path = args.path;
  let port = args.grpc_port;
  let mut atc_id = None;

  let sender = Sender::new(domain, port, token);
  let reader: Box<dyn TrackReader> = match args.service_type {
    ServiceType::TrackFile => {
      if let Some(arg_atc_id) = args.atc_id {
        atc_id = Some(arg_atc_id);
      }
      Box::new(TrackFileReader::new(&path))
    }
    ServiceType::Simwatch => {
      let mut reader = SimwatchReader::new(&path);
      reader.prepare().await?;
      atc_id = reader.atc_id();
      Box::new(reader)
    }
  };

  if let Some(atc_id) = atc_id {
    let flight_id = sender.create_flight().await?;
    println!("flight created with id {flight_id}");
    let rx = reader.read().await?;
    sender.send(&flight_id, &atc_id, rx).await?;
  } else {
    println!("atc_id is not defined")
  }

  Ok(())
}
