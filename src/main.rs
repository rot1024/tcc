use clap::Clap;
use std::error::Error;
use std::fs::File;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    io::stdout,
    io::Write,
    path::Path,
    str::FromStr,
};
use tcc::{analyze, csv_parser, markdown, Task};

fn main() -> Result<(), Box<dyn Error>> {
    let arg = App::parse();
    match arg.command {
        Command::Project(c) => c.project()?,
        Command::Analyze(c) => c.analyze()?,
    };
    Ok(())
}

#[derive(Debug, Clap)]
#[clap(name = "tcc", author, about, version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clap)]
pub enum Command {
    /// Show project names and IDs
    #[clap(name = "project")]
    Project(ProjectCommand),
    /// Extract tasks of a specified project and calculate used time
    #[clap(name = "analyze")]
    Analyze(AnalyzeCommand),
}

#[derive(Debug, Clap)]
#[clap(name = "project")]
pub struct ProjectCommand {
    file: String,
}

impl ProjectCommand {
    pub fn project(&self) -> Result<(), Box<dyn Error>> {
        let tasks = load(&self.file)?;

        let projects: HashSet<_> = tasks.into_iter().filter_map(|t| t.project).collect();

        projects
            .into_iter()
            .for_each(|p| println!("{} - {}", p.id, p.name));

        Ok(())
    }
}

#[derive(Debug, Clap)]
#[clap(name = "analyze")]
pub struct AnalyzeCommand {
    file: String,
    /// Target project ID
    #[clap(short, long)]
    project: String,
    /// Format: markdown, json
    #[clap(short, long, default_value = "markdown")]
    format: Format,
}

impl AnalyzeCommand {
    pub fn analyze(&self) -> Result<(), Box<dyn Error>> {
        let tasks = load(&self.file)?;
        let res = analyze(tasks, &self.project).expect("Project is not found.");

        match self.format {
            Format::JSON => {
                serde_json::to_writer(stdout(), &res)?;
            }
            Format::Markdown => {
                let out = stdout();
                let mut stdout = out.lock();
                markdown::write_to(&mut stdout, &res)?;
                stdout.flush()?;
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum Format {
    Markdown,
    JSON,
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Self::Markdown => "markdown",
                &Self::JSON => "json",
            }
        )
    }
}

impl FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "markdown" => Ok(Self::Markdown),
            "md" => Ok(Self::Markdown),
            "json" => Ok(Self::JSON),
            _ => Err("invalid format"),
        }
    }
}

fn load<P: AsRef<Path>>(file_name: P) -> Result<Vec<Task>, Box<dyn Error>> {
    let file = File::open(file_name)?;
    Ok(csv_parser::parse(&file))
}
