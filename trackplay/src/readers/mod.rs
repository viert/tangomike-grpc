pub mod simwatch;
pub mod trackfile;

use async_trait::async_trait;
use tm_grpc::service::tangomike::TrackMessage;
use tokio::sync::mpsc::Receiver;

#[async_trait]
pub trait TrackReader {
  async fn read(&self) -> Result<Receiver<TrackMessage>, Box<dyn std::error::Error>>;
}
