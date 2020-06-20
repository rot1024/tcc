use chrono::{Duration, NaiveDateTime};
use clap::Clap;
use serde_derive::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::Path;

mod loader;
mod model;
mod opt;

#[derive(Debug, Serialize)]
struct Task {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_time: Option<i64>,
    time_gap_ratio: Option<f64>,
    begin_time: NaiveDateTime,
    end_time: NaiveDateTime,
    timespan: i64,
}

#[derive(Debug, Serialize)]
struct AnalyzedResult {
    /// 合計見積時間
    total_estimated_time: i64,
    /// 合計所要時間
    total_used_time: i64,
    /// 合計見積時間と合計所要時間の倍率
    total_time_gap_ratio: f64,
    /// タスクを開始してから最後のタスクが終わるまでにかかった日数
    total_period_days: i64,
    /// 一日あたりの所要時間
    used_time_per_day: f64,
    tasks: Vec<Task>,
}

impl AnalyzedResult {
    fn new(tasks: &Vec<&model::Task>) -> AnalyzedResult {
        let total_estimated_time = tasks
            .iter()
            .filter_map(|t| t.estimated_time.map(|tt| tt.num_minutes()))
            .sum();
        let total_used_time = tasks
            .iter()
            .filter_map(|t| {
                t.begin_time
                    .and_then(|bt| t.end_time.map(|et| (et - bt).num_minutes()))
            })
            .sum();
        let total_period_days = tasks
            .first()
            .and_then(|f| tasks.last().map(|l| (f, l)))
            .and_then(|(f, l)| f.begin_time.and_then(|ft| l.end_time.map(|et| et - ft)))
            .map(|d| d.num_days())
            .unwrap_or(0);

        AnalyzedResult {
            total_estimated_time,
            total_used_time,
            total_time_gap_ratio: total_used_time as f64 / total_estimated_time as f64,
            total_period_days,
            used_time_per_day: total_used_time as f64 / total_period_days as f64,
            tasks: tasks
                .iter()
                .filter(|t| t.begin_time.and(t.end_time).is_some())
                .map(|t| Task {
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
                .collect(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let arg = opt::App::parse();

    match arg.command {
        opt::Command::Project { file } => {
            let tasks = load(file)?;

            let projects: HashSet<_> = tasks.into_iter().filter_map(|t| t.project).collect();

            projects
                .into_iter()
                .for_each(|p| println!("{} - {}", p.id, p.name));
        }
        opt::Command::Analyze {
            file,
            format,
            project,
        } => {
            analyze(&file, &project, format)?;
        }
    };

    Ok(())
}

fn analyze(file_name: &str, project_id: &str, format: opt::Format) -> Result<(), Box<dyn Error>> {
    let tasks = &load(file_name)?;
    let project_name = tasks
        .into_iter()
        .find(|t| {
            t.project
                .as_ref()
                .map(|p| p.id == project_id)
                .unwrap_or(false)
        })
        .map(|p| p.project.as_ref().unwrap().name.to_string())
        .expect("project is not found");

    let target_tasks: Vec<_> = tasks
        .into_iter()
        .filter(|t| {
            t.project
                .as_ref()
                .map(|p| p.id == project_id)
                .unwrap_or(false)
        })
        .collect();

    let res = AnalyzedResult::new(&target_tasks);

    match format {
        opt::Format::JSON => {
            serde_json::to_writer(stdout(), &res)?;
        }
        opt::Format::Markdown => {
            let out = stdout();
            let mut stdout = out.lock();
            write!(
                stdout,
                r#"# Review - {}

|タスク|日付|開始時刻|終了時刻|予定|実績|実績/予定|コメント|
|---|---|---|---|---|---|---|---|
"#,
                project_name
            )?;
            for t in res.tasks.into_iter() {
                write!(
                    stdout,
                    "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
                    t.name,
                    t.begin_time.date().format("%Y-%m-%d"),
                    t.begin_time.time().format("%H:%M"),
                    t.end_time.time().format("%H:%M"),
                    t.estimated_time
                        .map(|e| e.to_string())
                        .unwrap_or("-".to_string()),
                    t.timespan,
                    t.time_gap_ratio
                        .map(|r| format!("{:.2}", r))
                        .unwrap_or("-".to_string()),
                    t.comment.unwrap_or(String::new())
                )?;
            }

            let estimated_time = Duration::minutes(res.total_estimated_time);
            let used_time = Duration::minutes(res.total_used_time);
            let per_day = Duration::minutes(res.used_time_per_day as i64);

            write!(
                stdout,
                r#"
- 合計見積時間: {}h ({:.2}d)
- 合計所要時間: {}h ({:.2}d) (x{:.2})
- 実施期間： {}日
- 1日あたり所要時間： {:02}:{:02}
"#,
                estimated_time.num_hours(),
                estimated_time.num_hours() as f64 / 24.0,
                used_time.num_hours(),
                used_time.num_hours() as f64 / 24.0,
                res.total_time_gap_ratio,
                res.total_period_days,
                per_day.num_hours(),
                per_day.num_minutes() % 60
            )?;
            stdout.flush()?;
        }
    };
    Ok(())
}

fn load<P: AsRef<Path>>(file_name: P) -> Result<Vec<model::Task>, Box<dyn Error>> {
    let file = File::open(file_name)?;
    Ok(loader::load_taskchute_tsv(&file))
}
