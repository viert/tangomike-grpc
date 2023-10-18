pub mod meta;
pub mod tangomike {
  tonic::include_proto!("tangomike");
}
use self::{
  meta::FlightMeta,
  tangomike::{
    track_message, track_server::Track, track_stream_request::Union, EchoResponse, TrackRequest,
    TrackResponse, TrackStreamAck, TrackStreamRequest, TrackStreamResponse,
  },
};
use crate::{
  geodata::GeoData,
  track::{entry::TrackFileEntry, store::TrackStore},
  util::proxy_requests,
};
use chrono::Utc;
use log::{error, info};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::{
  sync::mpsc::{self, error::TryRecvError},
  time::sleep,
};
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct TrackService {
  geo: Arc<GeoData>,
  store: TrackStore,
}

impl TrackService {
  pub fn new(geo: GeoData, store: TrackStore) -> Self {
    Self {
      geo: Arc::new(geo),
      store,
    }
  }
}

#[tonic::async_trait]
impl Track for TrackService {
  #[doc = " Server streaming response type for the TrackStream method."]
  type TrackStreamStream =
    Pin<Box<dyn Stream<Item = Result<TrackStreamResponse, Status>> + Send + 'static>>;

  async fn track_stream(
    &self,
    request: Request<Streaming<TrackStreamRequest>>,
  ) -> Result<Response<Self::TrackStreamStream>, Status> {
    let remote = request.remote_addr().unwrap();
    let remote = format!("track_stream:{:?}", remote);
    info!("[{remote}] client connected");

    let meta: FlightMeta = request.metadata().try_into()?;
    let mut seq_number = 1;

    let stream = request.into_inner();
    let mut tf = self.store.open_or_create(&meta.flight_id)?;
    let geo = self.geo.clone();

    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move { proxy_requests(stream, tx).await });

    let output = async_stream::try_stream! {
      let mut rx = rx;
      loop {
        let res = rx.try_recv();
        match res {
          Err(TryRecvError::Disconnected) => {
            info!("received disconnected error");
            break
          },
          Err(TryRecvError::Empty) => {
            sleep(Duration::from_millis(10)).await;
          },
          Ok(msg) => {
            match msg.union.unwrap() {
              Union::TrackMessage(msg) => {
                let union = &msg.union;

                match union {
                  Some(union) => match union {
                    track_message::Union::Point(pt) => {
                      if pt.on_gnd {
                        let dep = tf.get_departure()?;
                        if dep.is_empty() {
                          let closest = geo.closest_airport(pt.lng, pt.lat);
                          if let Some(arpt) = closest {
                            tf.set_departure(&arpt.ident)?;
                          }
                        }
                      }
                    },
                    track_message::Union::TouchDown(td) => {
                      let arr = tf.get_arrival()?;
                      if arr.is_empty() {
                        let closest = geo.closest_airport(td.lng, td.lat);
                        if let Some(arpt) = closest {
                          tf.set_arrival(&arpt.ident)?;
                        }
                      }
                    }
                  },
                  None => {
                    error!("got an empty request message");
                    break;
                  }
                }

                let entry: TrackFileEntry = msg.into();
                tf.append(&entry)?;
                let msg = TrackStreamResponse {
                  ack: Some(TrackStreamAck {
                    seq_number,
                    echo_response: None
                  })
                };
                yield msg;
              },
              Union::EchoRequest(req) => {
                let client_ts = req.timestamp_us;
                let server_ts = Utc::now().timestamp_micros() as u64;
                let resp = EchoResponse {
                  client_timestamp_us: client_ts,
                  server_timestamp_us: server_ts,
                };
                let msg = TrackStreamResponse {
                  ack: Some(TrackStreamAck {
                    seq_number,
                    echo_response: Some(resp)
                  })
                };
                yield msg;
              },
            }
            seq_number += 1;
          }
        }
      }
      info!("[{remote}] client disconnected");
    };

    Ok(Response::new(Box::pin(output) as Self::TrackStreamStream))
  }

  async fn get_track(
    &self,
    request: Request<TrackRequest>,
  ) -> Result<Response<TrackResponse>, Status> {
    let req = request.into_inner();
    let tf = self.store.open(&req.flight_id)?;

    let entries = tf.read_all()?;
    let mut points = vec![];
    let mut touchdowns = vec![];

    for entry in entries {
      match entry {
        TrackFileEntry::TrackPoint(pt) => points.push(pt),
        TrackFileEntry::TouchDown(td) => touchdowns.push(td),
      }
    }

    let resp = TrackResponse {
      points: points.into_iter().map(|p| p.into()).collect(),
      touchdowns: touchdowns.into_iter().map(|t| t.into()).collect(),
    };

    Ok(Response::new(resp))
  }
}
