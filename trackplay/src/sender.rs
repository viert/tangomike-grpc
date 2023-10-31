use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use std::time::Duration;
use tm_grpc::service::tangomike::{self, TrackMessage, UploadTrackStreamRequest};
use tokio::{
  sync::mpsc::{error::TryRecvError, Receiver},
  time::sleep,
};
use tonic::{metadata::MetadataValue, Request};

pub struct Sender {
  token: String,
  domain: String,
  port: u16,
}

#[derive(Deserialize)]
struct CreateFlightResponse {
  pub flight_id: String,
}

impl Sender {
  pub fn new(domain: String, port: u16, token: String) -> Self {
    Self {
      token,
      domain,
      port,
    }
  }

  pub async fn create_flight(&self) -> Result<String, Box<dyn std::error::Error>> {
    let cli = reqwest::Client::new();
    let url = format!("https://{}/api/v1/flights/", self.domain);
    let auth_header = format!("Token {}", self.token);
    let resp = cli
      .post(url)
      .header(AUTHORIZATION, auth_header)
      .send()
      .await?;
    let resp = resp.json::<CreateFlightResponse>().await?;
    Ok(resp.flight_id)
  }

  pub async fn send(
    &self,
    flight_id: &str,
    atc_id: &str,
    rx: Receiver<TrackMessage>,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let dst = format!("http://{}:{}", self.domain, self.port);
    let mut client = tangomike::track_client::TrackClient::connect(dst).await?;

    let outbound = async_stream::stream! {
      let mut rx = rx;
      let mut idx = 1;
      loop {
        let res = rx.try_recv();
        match res {
          Err(TryRecvError::Disconnected) => break,
          Err(TryRecvError::Empty) => {
            sleep(Duration::from_millis(10)).await;
          }
          Ok(msg) => {
            let req = UploadTrackStreamRequest {
              request_id: idx,
              union: Some(tangomike::upload_track_stream_request::Union::TrackMessage(
                msg
              )),
            };
            yield req;
            idx += 1;
          }
        }
      }
    };

    let mut request = Request::new(outbound);
    let meta = request.metadata_mut();
    meta.append("x-flight-id", MetadataValue::try_from(flight_id)?);
    meta.append("x-atc-id", MetadataValue::try_from(atc_id)?);
    meta.append("x-auth-token", MetadataValue::try_from(&self.token)?);

    let response = client.upload_track_stream(request).await?;
    let mut inbound = response.into_inner();

    while let Some(resp) = inbound.message().await? {
      if let Some(ack) = resp.ack {
        println!("ack received: {}", ack.request_id);
      }
    }
    Ok(())
  }
}
