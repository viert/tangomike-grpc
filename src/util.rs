use chrono::{DateTime, Utc};
use log::error;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;
use tonic::Streaming;

pub async fn proxy_requests<T>(mut stream: Streaming<T>, tx: Sender<T>) {
  while let Some(msg) = stream.next().await {
    if let Ok(msg) = msg {
      let res = tx.send(msg).await;
      if let Err(err) = res {
        error!("error sending request via channel: {err:?}");
        break;
      }
    }
  }
}

pub fn seconds_since(t: DateTime<Utc>) -> f32 {
  let t2 = Utc::now();
  let d = (t2 - t).to_std();
  if let Ok(d) = d {
    d.as_secs_f32()
  } else {
    0.0
  }
}
