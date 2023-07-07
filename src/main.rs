use std::path::PathBuf;

use chrono::{Duration, Local};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};

mod state;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Opts {
    #[command(subcommand)]
    command: Command,

    #[arg(long, env = "MONTAGE_SCRIPTS", global = true)]
    scripts: Option<PathBuf>,
}

impl Opts {
    fn run(&self) -> Result<()> {
        let mut store = state::Store::create_or_load()?;

        match &self.command {
            Command::Start { name, duration } => {
                store.start(
                    name.to_string(),
                    Local::now() + Duration::minutes(TryInto::try_into(*duration)?),
                );
                store
                    .write()
                    .wrap_err("could not write state after starting")?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Start a task
    Start {
        /// The name of the task you're working on
        name: String,

        /// How long you're planning to work, in minutes
        #[arg(long, default_value = "25")]
        duration: usize,
    },

    /// Take a break in between tasks
    Break {
        /// How long you're going to rest, in minutes
        #[arg(long, default_value = "5")]
        duration: usize,
    },

    /// Show an xbar status message
    Xbar,

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Vex,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    Opts::parse().run()?;

    Ok(())
}
