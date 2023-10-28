pub mod meta;
pub mod tangomike {
  tonic::include_proto!("tangomike");
}
mod state;

use self::{
  meta::FlightMeta,
  state::ServiceState,
  tangomike::{
    track_message, track_server::Track, upload_track_stream_request::Union, ActiveFlightsResponse,
    DownloadTrackStreamRequest, EchoResponse, NoParams, TrackMessage, TrackRequest, TrackResponse,
    UploadTrackStreamAck, UploadTrackStreamRequest, UploadTrackStreamResponse,
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
  sync::{
    mpsc::{self, error::TryRecvError},
    RwLock,
  },
  time::sleep,
};
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct TrackService {
  geo: Arc<GeoData>,
  store: TrackStore,
  state: Arc<RwLock<ServiceState>>,
}

impl TrackService {
  pub fn new(geo: GeoData, store: TrackStore) -> Self {
    Self {
      geo: Arc::new(geo),
      store,
      state: Arc::new(RwLock::new(Default::default())),
    }
  }
}

#[tonic::async_trait]
impl Track for TrackService {
  #[doc = " Server streaming response type for the TrackStream method."]
  type UploadTrackStreamStream =
    Pin<Box<dyn Stream<Item = Result<UploadTrackStreamResponse, Status>> + Send + 'static>>;
  type DownloadTrackStreamStream =
    Pin<Box<dyn Stream<Item = Result<TrackMessage, Status>> + Send + 'static>>;

  async fn download_track_stream(
    &self,
    request: Request<DownloadTrackStreamRequest>,
  ) -> Result<Response<Self::DownloadTrackStreamStream>, Status> {
    let req = request.into_inner();
    let tf = self.store.open(&req.flight_id)?;
    let output = async_stream::try_stream! {
      let mut count = tf.count()? as usize;
      let mut idx = 0;
      while idx < count {
        let entry = tf.read_at(idx)?;

        let ts = match &entry {
          TrackFileEntry::TrackPoint(tp) => tp.ts,
          TrackFileEntry::TouchDown(td) => td.ts,
        };

        if ts > req.start_at {
          yield entry.into();
        }

        idx += 1;
      }

      loop {
        let new_count = tf.count()? as usize;
        if new_count > count {
          count = new_count;
          while idx < count {
            let entry = tf.read_at(idx)?;
            yield entry.into();
            idx += 1;
          }
        }
        sleep(Duration::from_secs(1)).await;
      }
    };

    Ok(Response::new(
      Box::pin(output) as Self::DownloadTrackStreamStream
    ))
  }

  async fn upload_track_stream(
    &self,
    request: Request<Streaming<UploadTrackStreamRequest>>,
  ) -> Result<Response<Self::UploadTrackStreamStream>, Status> {
    let remote = request.remote_addr().unwrap();
    let remote = format!("track_stream:{:?}", remote);
    info!("[{remote}] client connected");

    let meta: FlightMeta = request.metadata().try_into()?;

    let stream = request.into_inner();
    let mut tf = self.store.open_or_create(&meta.flight_id)?;
    let geo = self.geo.clone();

    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move { proxy_requests(stream, tx).await });

    let state = self.state.clone();

    let output = async_stream::try_stream! {
      state.write().await.add_active_flight(&meta.flight_id);

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
            let request_id = msg.request_id;
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
                let msg = UploadTrackStreamResponse {
                  ack: Some(UploadTrackStreamAck {
                    request_id,
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
                let msg = UploadTrackStreamResponse {
                  ack: Some(UploadTrackStreamAck {
                    request_id,
                    echo_response: Some(resp)
                  })
                };
                yield msg;
              },
            }
          }
        }
      }
      info!("[{remote}] client disconnected");
      state.write().await.remove_active_flight(&meta.flight_id);
    };

    Ok(Response::new(
      Box::pin(output) as Self::UploadTrackStreamStream
    ))
  }

  async fn get_active_flights(
    &self,
    _: Request<NoParams>,
  ) -> Result<Response<ActiveFlightsResponse>, Status> {
    let flight_ids: Vec<String> = self
      .state
      .write()
      .await
      .active_flights
      .iter()
      .cloned()
      .collect();
    Ok(Response::new(ActiveFlightsResponse { flight_ids }))
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

    let header = tf.get_header()?;

    let resp = TrackResponse {
      flight_id: header.get_flight_id(),
      departure: header.get_departure(),
      arrival: header.get_arrival(),
      points: points.into_iter().map(|p| p.into()).collect(),
      touchdowns: touchdowns.into_iter().map(|t| t.into()).collect(),
    };

    Ok(Response::new(resp))
  }
}
