use crate::AnalysisResult;
use chrono::Duration;
use std::{error::Error, io::Write};

pub fn write_to<W: Write>(w: &mut W, v: &AnalysisResult) -> Result<(), Box<dyn Error>> {
    write!(
        w,
        r#"# Review - {}

|タスク|日付|開始時刻|終了時刻|予定|実績|実績/予定|コメント|
|---|---|---|---|---|---|---|---|
"#,
        v.project_name
    )?;

    for t in v.tasks.iter() {
        write!(
            w,
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
            t.comment.clone().unwrap_or(String::new())
        )?;
    }

    let estimated_time = Duration::minutes(v.total_estimated_time);
    let used_time = Duration::minutes(v.total_used_time);
    let per_day = Duration::minutes(v.used_time_per_day as i64);

    write!(
        w,
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
        v.total_time_gap_ratio,
        v.total_period_days,
        per_day.num_hours(),
        per_day.num_minutes() % 60
    )?;

    Ok(())
}
