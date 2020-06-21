use crate::{analyzer::TasksAnalysisResult, AnalysisResult, AnalysisResultTask};
use chrono::Duration;
use std::{
    error::Error,
    fmt::{self, Display},
    io::Write,
};

pub fn write_to<W: Write>(w: &mut W, v: &AnalysisResult) -> Result<(), Box<dyn Error>> {
    write!(
        w,
        r#"# {}

{}
## 全タスクの分析

{}
"#,
        v.project_name,
        Tasks(&v.all.tasks),
        Analysis(&v.all, v.value)
    )?;

    Ok(())
}

struct Tasks<'a>(&'a [AnalysisResultTask]);

impl<'a> Tasks<'a> {
    const HEADERS: [&'static str; 8] = [
        "タスク",
        "日付",
        "開始時刻",
        "終了時刻",
        "予定",
        "実績",
        "実績/予定",
        "コメント",
    ];
}

impl<'a> Display for Tasks<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "|{}|\n|{}|\n",
            Self::HEADERS.join("|"),
            Self::HEADERS
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join("|")
        )?;

        for t in self.0.iter() {
            write!(
                f,
                "|{name}|{date}|{begin}|{end}|{estimated}|{timespan}|{gap}|{comment}|\n",
                name = t.name,
                date = t.begin_time.date().format("%Y-%m-%d"),
                begin = t.begin_time.time().format("%H:%M"),
                end = t.end_time.time().format("%H:%M"),
                estimated = t
                    .estimated_time
                    .map(|e| e.to_string())
                    .unwrap_or("-".to_string()),
                timespan = t.timespan,
                gap = t
                    .time_gap_ratio
                    .map(|r| format!("{:.2}", r))
                    .unwrap_or("-".to_string()),
                comment = t.comment.clone().unwrap_or(String::new())
            )?;
        }

        Ok(())
    }
}

struct Analysis<'a>(&'a TasksAnalysisResult, Option<i64>);

impl<'a> Display for Analysis<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let estimated_time = Timespan::from(self.0.total_estimated_time);
        let work_time = Timespan::from(self.0.total_work_time);
        let (per_day, min, max, median, deviation, per_value) = (
            Timespan::from(self.0.work_time_per_day),
            Timespan::from(self.0.work_time_per_day_min),
            Timespan::from(self.0.work_time_per_day_max),
            Timespan::from(self.0.work_time_per_day_median),
            Timespan::from(self.0.work_time_per_day_deviation),
            self.0.work_time_per_value.map(Timespan::from),
        );

        write!(
            f,
            r#"- 合計見積時間: {estimated_time}
- 合計所要時間: {work_time}{gap}
- 実施期間： {period}d
- 1日あたり所要時間
    - 平均：{per_day}
    - 最大：{max}
    - 中央：{median}
    - 最小：{min}
    - 標準偏差：{deviation}{per_value}"#,
            estimated_time = estimated_time,
            work_time = work_time,
            gap = self
                .0
                .total_time_gap_ratio
                .map(|r| format!(" (x{:.2})", r))
                .unwrap_or_default(),
            period = self.0.work_days,
            per_day = per_day,
            min = min,
            max = max,
            median = median,
            deviation = deviation,
            per_value = per_value
                .and_then(|v| self.1.map(|w| (v, w)))
                .map(|(v, w)| format!("\n- 1ページあたりの所要時間： {} （全{}ページ）", v, w))
                .unwrap_or_default(),
        )
    }
}

struct Timespan(Duration);

impl From<i64> for Timespan {
    fn from(d: i64) -> Self {
        Self(Duration::minutes(d))
    }
}

impl From<f64> for Timespan {
    fn from(d: f64) -> Self {
        Self(Duration::seconds(d as i64 * 60))
    }
}

impl Display for Timespan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_zero() {
            return write!(f, "-");
        };

        let mut r = Vec::<String>::new();

        let (w, d, h, m, s) = (
            self.0.num_weeks(),
            self.0.num_days(),
            self.0.num_hours(),
            self.0.num_minutes(),
            self.0.num_seconds(),
        );

        if w > 0 {
            r.push(format!(
                "{:02}w ({}w{}d)",
                ceil(w as f64 + ((d % 7) as f64 / 7f64), 2),
                w,
                d % 27
            ))
        }
        if d > 0 {
            r.push(format!(
                "{:02}d ({}d{}h)",
                ceil(d as f64 + ((h % 24) as f64 / 24f64), 2),
                d,
                (h % 24)
            ))
        }
        if h > 0 {
            r.push(format!(
                "{:02}h ({}h{}m)",
                ceil(h as f64 + ((m % 60) as f64 / 60f64), 2),
                h,
                m % 60
            ))
        }
        if m > 0 {
            if s % 60 != 0 {
                r.push(format!(
                    "{:02}m ({}m{}s)",
                    ceil(m as f64 + ((s % 60) as f64 / 60f64), 2),
                    m,
                    s % 60
                ))
            } else {
                r.push(format!("{}m", m))
            }
        }
        if s % 60 != 0 {
            r.push(format!("{}s", s))
        }

        write!(f, "{}", r.join(" = "))
    }
}

pub fn ceil(value: f64, scale: i8) -> f64 {
    let multiplier = 10f64.powi(scale as i32) as f64;
    (value * multiplier).ceil() / multiplier
}
