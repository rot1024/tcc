use clap::Clap;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

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
    Project { file: String },

    /// Extract tasks of a specified project and calculate used time
    #[clap(name = "analyze")]
    Analyze {
        file: String,
        /// Target project ID
        #[clap(short, long)]
        project: String,
        /// Format: markdown, json
        #[clap(short, long, default_value = "markdown")]
        format: Format,
    },
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
