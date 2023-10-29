use reqwest::{header::AUTHORIZATION, Client};

#[derive(Debug)]
pub struct ApiConnect {
  base_uri: String,
}

impl ApiConnect {
  pub fn new(base_uri: &str) -> Self {
    Self {
      base_uri: base_uri.into(),
    }
  }

  pub async fn check_flight_id(
    &self,
    flight_id: &str,
    auth_token: &str,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    let url = format!("{}/api/v1/flights/{flight_id}/check", self.base_uri);
    let client = Client::new();
    let res = client
      .get(url)
      .header(AUTHORIZATION, format!("Token {auth_token}"))
      .send()
      .await?;
    let status = res.status();
    Ok(status.is_success())
  }
}
