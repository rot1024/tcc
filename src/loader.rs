use chrono::{Duration, NaiveDate, NaiveTime};
use encoding_rs_io::DecodeReaderBytes;
use serde_derive::Deserialize;
use std::error::Error;
use std::io::Read;

use super::model;

#[derive(Deserialize, Debug)]
struct TaskDTO {
  #[serde(rename = "実行日")]
  date: String,
  #[serde(rename = "タスク名")]
  name: String,
  #[serde(rename = "見積時間")]
  estimated_time: Option<String>,
  #[serde(rename = "実績時間")]
  used_time: Option<String>,
  #[serde(rename = "開始時間")]
  begin_time: Option<String>,
  #[serde(rename = "終了時間")]
  end_time: Option<String>,
  #[serde(rename = "コメント")]
  comment: Option<String>,
  #[serde(rename = "プロジェクト名")]
  project_name: Option<String>,
  #[serde(rename = "プロジェクトID")]
  project_id: Option<String>,
}

impl TaskDTO {
  pub fn to_task(&self) -> Result<model::Task, Box<dyn Error>> {
    let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")?;
    let estimated_time = self
      .estimated_time
      .as_ref()
      .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok())
      .map(|t| t - NaiveTime::from_hms(0, 0, 0))
      .and_then(|t| if t.num_minutes() == 0 { None } else { Some(t) });
    let begin_time = self
      .begin_time
      .as_ref()
      .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok())
      .map(|t| date.and_time(t));
    let end_time = self
      .end_time
      .as_ref()
      .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok())
      .map(|t| {
        date.and_time(t)
          + Duration::days(
            begin_time
              .map(|bt| if t < bt.time() { 1 } else { 0 })
              .unwrap_or(0),
          )
      });
    let project = self
      .project_name
      .as_ref()
      .and_then(|n| self.project_id.as_ref().map(|p| (p, n)));

    Ok(model::Task {
      name: self.name.to_string(),
      estimated_time,
      begin_time,
      end_time,
      comment: self.comment.clone(),
      project: project.map(|(p, n)| model::Project {
        name: n.to_string(),
        id: p.to_string(),
      }),
    })
  }
}

pub fn load_taskchute_tsv(r: impl Read) -> Vec<model::Task> {
  csv::ReaderBuilder::new()
    .delimiter(b'\t')
    .has_headers(true)
    .from_reader(DecodeReaderBytes::new(r))
    .deserialize::<TaskDTO>()
    .into_iter()
    .filter_map(|r| r.ok().and_then(|t| t.to_task().ok()))
    .collect()
}
