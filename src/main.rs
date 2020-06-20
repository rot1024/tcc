use clap::Clap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::Path;
use tcc::{AnalysisResult, Task};

mod loader;
mod markdown;
mod opt;

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

    let res = AnalysisResult::new(&target_tasks, project_name);

    match format {
        opt::Format::JSON => {
            serde_json::to_writer(stdout(), &res)?;
        }
        opt::Format::Markdown => {
            let out = stdout();
            let mut stdout = out.lock();
            markdown::write_to(&mut stdout, &res)?;
            stdout.flush()?;
        }
    };
    Ok(())
}

fn load<P: AsRef<Path>>(file_name: P) -> Result<Vec<Task>, Box<dyn Error>> {
    let file = File::open(file_name)?;
    Ok(loader::load_taskchute_tsv(&file))
}
