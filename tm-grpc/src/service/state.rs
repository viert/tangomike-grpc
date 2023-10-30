use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct ServiceState {
  pub active_flights: HashSet<String>,
}

impl ServiceState {
  pub fn add_active_flight(&mut self, flight_id: &str) {
    self.active_flights.insert(flight_id.into());
  }

  pub fn remove_active_flight(&mut self, flight_id: &str) {
    self.active_flights.remove(flight_id);
  }

  pub fn is_active(&self, flight_id: &str) -> bool {
    self.active_flights.contains(flight_id)
  }
}
