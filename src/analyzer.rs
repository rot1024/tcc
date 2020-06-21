use crate::{Project, Task};
use chrono::NaiveDateTime;
use itertools::Itertools;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub project_name: String,
    pub value: Option<i64>,
    pub all: TasksAnalysisResult,
    // 平日・休日別
    // pub day: HashMap<String, TasksAnalysisResult>,
    // 曜日別
    // pub weekday: HashMap<String, TasksAnalysisResult>,
    // グループ別
    // pub group: HashMap<String, TasksAnalysisResult>,
}

pub fn analyze(tasks: Vec<Task>, project_id: &str, value: Option<i64>) -> Option<AnalysisResult> {
    let target_tasks = Tasks::new(tasks, project_id, value);
    let project_name = target_tasks.project_name(project_id)?;

    Some(AnalysisResult {
        project_name,
        value,
        all: target_tasks.analyze(),
    })
}

#[derive(Debug, Serialize, Clone)]
pub struct AnalysisResultTask {
    pub id: String,
    pub name: String,
    pub group: Option<String>,
    pub project: Option<Project>,
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

impl PartialEq for AnalysisResultTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AnalysisResultTask {}

impl Ord for AnalysisResultTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.begin_time.cmp(&other.begin_time)
    }
}

impl PartialOrd for AnalysisResultTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.begin_time.cmp(&other.begin_time))
    }
}

impl From<Task> for AnalysisResultTask {
    fn from(task: Task) -> Self {
        let begin_time = task.begin_time.unwrap();
        let end_time = task.end_time.unwrap();

        Self {
            id: task.id,
            name: task.name,
            group: None,
            project: task.project,
            comment: task.comment,
            estimated_time: task.estimated_time.map(|t| t.num_minutes()),
            time_gap_ratio: task
                .estimated_time
                .map(|e| (end_time - begin_time).num_minutes() as f64 / e.num_minutes() as f64),
            begin_time: task.begin_time.unwrap(),
            end_time: task.end_time.unwrap(),
            timespan: (task.end_time.unwrap() - task.begin_time.unwrap()).num_minutes(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Tasks {
    tasks: Vec<AnalysisResultTask>,
    value: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TasksAnalysisResult {
    /// 合計見積時間
    pub total_estimated_time: i64,
    /// 合計作業時間標
    pub total_work_time: i64,
    /// 合計見積時間と合計所要時間の倍率
    pub total_time_gap_ratio: Option<f64>,
    /// 稼働日数（1分でも稼働したらその日は稼働したとしてカウント）
    pub work_days: i64,
    /// 1日あたり作業時間平均
    pub work_time_per_day: f64,
    /// 1日あたり作業時間最大
    pub work_time_per_day_max: i64,
    /// 1日あたり作業時間最小
    pub work_time_per_day_min: i64,
    /// 1日あたり作業時間中央
    pub work_time_per_day_median: i64,
    /// 1日あたり作業時間標準偏差
    pub work_time_per_day_deviation: f64,
    /// 1ページあたりの作業時間（ページ数といったパラメータを外から差し込む）
    pub work_time_per_value: Option<f64>,
    /// 作業別（タスクごとの所要時間を並べる）
    pub tasks: Vec<AnalysisResultTask>,
}

impl Tasks {
    fn new<T: IntoIterator<Item = Task>>(tasks: T, project_id: &str, value: Option<i64>) -> Self {
        Self {
            tasks: tasks
                .into_iter()
                .filter(|t| {
                    t.project
                        .as_ref()
                        .map(|p| p.id == project_id)
                        .unwrap_or(false)
                })
                .filter(|t| t.begin_time.and(t.end_time).is_some())
                .map(From::from)
                .sorted()
                .collect(),
            value,
        }
    }

    fn project_name(&self, project_id: &str) -> Option<String> {
        self.tasks
            .iter()
            .find(|t| {
                t.project
                    .as_ref()
                    .map(|p| p.id == project_id)
                    .unwrap_or(false)
            })
            .map(|p| p.project.as_ref().unwrap().name.to_string())
    }

    fn total_estimated_time(&self) -> i64 {
        self.tasks.iter().filter_map(|t| t.estimated_time).sum()
    }

    fn total_work_time(&self) -> i64 {
        self.tasks
            .iter()
            .map(|t| (t.end_time - t.begin_time).num_minutes())
            .sum()
    }

    fn work_days(&self) -> i64 {
        self.tasks
            .first()
            .and_then(|f| self.tasks.last().map(|l| (f, l)))
            .map(|(f, l)| (l.end_time - f.begin_time).num_days())
            .unwrap_or(0)
    }

    fn work_time_per_day(&self) -> f64 {
        self.total_work_time() as f64 / self.work_days() as f64
    }

    fn work_time_per_day_max(&self) -> i64 {
        self.tasks.iter().map(|t| t.timespan).max().unwrap_or(0)
    }

    fn work_time_per_day_min(&self) -> i64 {
        self.tasks.iter().map(|t| t.timespan).min().unwrap_or(0)
    }

    fn work_time_per_day_median(&self) -> i64 {
        let v: Vec<i64> = self.tasks.iter().map(|t| t.timespan).sorted().collect();
        v.get(v.len() / 2).map(|v| *v).unwrap_or(0)
    }

    fn work_time_per_day_deviation(&self) -> f64 {
        let a = self.work_time_per_day();
        (self
            .tasks
            .iter()
            .map(|t| (t.timespan as f64 - a).powi(2))
            .sum::<f64>()
            / self.tasks.len() as f64)
            .sqrt()
    }

    fn work_time_per_value(&self) -> Option<f64> {
        let v = self.value?;
        Some(self.total_work_time() as f64 / v as f64)
    }

    fn tasks(self) -> Vec<AnalysisResultTask> {
        self.tasks
    }

    fn analyze(self) -> TasksAnalysisResult {
        let tw = self.total_work_time();
        let te = self.total_estimated_time();

        TasksAnalysisResult {
            total_estimated_time: te,
            total_work_time: tw,
            total_time_gap_ratio: if te == 0 {
                None
            } else {
                Some(tw as f64 / te as f64)
            },
            work_days: self.work_days(),
            work_time_per_day: self.work_time_per_day(),
            work_time_per_day_max: self.work_time_per_day_max(),
            work_time_per_day_min: self.work_time_per_day_min(),
            work_time_per_day_median: self.work_time_per_day_median(),
            work_time_per_day_deviation: self.work_time_per_day_deviation(),
            work_time_per_value: self.work_time_per_value(),
            tasks: self.tasks(),
        }
    }
}
