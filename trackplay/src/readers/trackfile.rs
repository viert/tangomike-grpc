use std::time::Duration;

use super::TrackReader;
use async_trait::async_trait;
use chrono::Utc;
use tm_grpc::{
  service::tangomike::TrackMessage,
  track::{entry::TrackFileEntry, trackfile::TrackFile},
};
use tokio::{
  sync::mpsc::{self, Receiver},
  time::sleep,
};

pub struct TrackFileReader {
  path: String,
}

impl TrackFileReader {
  pub fn new(path: &str) -> Self {
    Self {
      path: path.to_owned(),
    }
  }
}

#[async_trait]
impl TrackReader for TrackFileReader {
  async fn read(&self) -> Result<Receiver<TrackMessage>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(1024);
    let tf = TrackFile::open(&self.path)?;
    let header = tf.get_header()?;

    let count = header.count();
    if count > 1 {
      let first = tf.read_at(0)?;
      let ts = match first {
        TrackFileEntry::TrackPoint(tp) => tp.ts,
        TrackFileEntry::TouchDown(td) => td.ts,
      };
      let now = Utc::now().timestamp_millis() as u64;
      let timediff = now - ts;

      tokio::spawn(async move {
        for i in 0..count {
          let res = tf.read_at(i as usize);
          if let Ok(entry) = res {
            let adj_now = Utc::now().timestamp_millis() as u64 - timediff;
            let ts = match &entry {
              TrackFileEntry::TrackPoint(tp) => tp.ts,
              TrackFileEntry::TouchDown(td) => td.ts,
            };

            if ts > adj_now {
              let sleep_time = Duration::from_millis(ts - adj_now);
              println!("sleeping for {sleep_time:?}");
              sleep(sleep_time).await;
            }

            let mut msg: TrackMessage = entry.into();
            msg.ts = Utc::now().timestamp_millis() as u64;
            let res = tx.send(msg).await;
            if let Err(err) = res {
              println!("Error sending entry: {err}");
            }
          } else {
            break;
          }
        }
      });
    }
    Ok(rx)
  }
}
