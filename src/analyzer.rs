use crate::Task;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub project_name: String,
    /// 合計見積時間
    pub total_estimated_time: i64,
    /// 合計所要時間
    pub total_used_time: i64,
    /// 合計見積時間と合計所要時間の倍率
    pub total_time_gap_ratio: f64,
    /// タスクを開始してから最後のタスクが終わるまでにかかった日数
    pub total_period_days: i64,
    /// 一日あたりの所要時間
    pub used_time_per_day: f64,
    pub tasks: Vec<AnalysisResultTask>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisResultTask {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_gap_ratio: Option<f64>,
    pub begin_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub timespan: i64,
}

pub fn analyze(tasks: Vec<Task>, project_id: &str) -> Option<AnalysisResult> {
    let project_name = tasks
        .iter()
        .find(|t| {
            t.project
                .as_ref()
                .map(|p| p.id == project_id)
                .unwrap_or(false)
        })
        .map(|p| p.project.as_ref().unwrap().name.to_string())?;

    let target_tasks = tasks
        .into_iter()
        .filter(|t| {
            t.project
                .as_ref()
                .map(|p| p.id == project_id)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let total_estimated_time = target_tasks
        .iter()
        .filter_map(|t| t.estimated_time.map(|tt| tt.num_minutes()))
        .sum();

    let total_used_time = target_tasks
        .iter()
        .filter_map(|t| {
            t.begin_time
                .and_then(|bt| t.end_time.map(|et| (et - bt).num_minutes()))
        })
        .sum();

    let total_period_days = target_tasks
        .first()
        .and_then(|f| target_tasks.last().map(|l| (f, l)))
        .and_then(|(f, l)| f.begin_time.and_then(|ft| l.end_time.map(|et| et - ft)))
        .map(|d| d.num_days())
        .unwrap_or(0);

    let tasks = target_tasks
        .iter()
        .filter(|t| t.begin_time.and(t.end_time).is_some())
        .map(|t| AnalysisResultTask {
            name: t.name.to_string(),
            comment: t.comment.clone(),
            estimated_time: t.estimated_time.map(|t| t.num_minutes()),
            time_gap_ratio: t.estimated_time.map(|e| {
                (t.end_time.unwrap() - t.begin_time.unwrap()).num_minutes() as f64
                    / e.num_minutes() as f64
            }),
            begin_time: t.begin_time.unwrap(),
            end_time: t.end_time.unwrap(),
            timespan: (t.end_time.unwrap() - t.begin_time.unwrap()).num_minutes(),
        })
        .collect();

    Some(AnalysisResult {
        project_name,
        total_estimated_time,
        total_used_time,
        total_time_gap_ratio: total_used_time as f64 / total_estimated_time as f64,
        total_period_days,
        used_time_per_day: total_used_time as f64 / total_period_days as f64,
        tasks,
    })
}
