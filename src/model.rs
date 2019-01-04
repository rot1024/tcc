use chrono::{Duration, NaiveDateTime};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Task {
  pub name: String,
  pub estimated_time: Option<Duration>,
  pub begin_time: Option<NaiveDateTime>,
  pub end_time: Option<NaiveDateTime>,
  pub comment: Option<String>,
  pub project: Option<Project>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Project {
  pub id: String,
  pub name: String,
}
