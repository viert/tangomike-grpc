use tonic::{metadata::MetadataMap, Status};

pub struct FlightMeta {
  pub auth_token: String,
  pub flight_id: String,
  pub atc_id: String,
  pub atc_type: Option<String>,
  pub atc_flight_number: Option<String>,
  pub aircraft_title: Option<String>,
}

fn extract_key(meta: &MetadataMap, key: &str) -> Result<String, Status> {
  match meta.get(key) {
    Some(value) => {
      let res = value.to_str();
      match res {
        Ok(value) => Ok(value.into()),
        Err(_) => Err(Status::invalid_argument(format!("invalid {key} header"))),
      }
    }
    None => Err(Status::invalid_argument(format!("{key} header is missing"))),
  }
}

impl TryFrom<&MetadataMap> for FlightMeta {
  type Error = Status;

  fn try_from(value: &MetadataMap) -> Result<Self, Self::Error> {
    let flight_id = extract_key(value, "x-flight-id")?;
    let atc_id = extract_key(value, "x-atc-id")?;
    let auth_token = extract_key(value, "x-auth-token")?;
    let atc_type = extract_key(value, "x-atc-type").ok();
    let atc_flight_number = extract_key(value, "x-atc-flight-number").ok();
    let aircraft_title = extract_key(value, "x-title").ok();
    Ok(Self {
      flight_id,
      atc_id,
      atc_type,
      atc_flight_number,
      aircraft_title,
      auth_token,
    })
  }
}
