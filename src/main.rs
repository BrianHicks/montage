use chrono::{Duration, Local};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use rand::seq::SliceRandom;

mod scripts;
mod state;

static THINGS_TO_SAY: [&'static str; 4] = [
    "hey",
    "pick a new task",
    "Brian",
    "time for a break?"
];

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Opts {
    #[command(subcommand)]
    command: Command,

    #[arg(long, env = "MONTAGE_SCRIPTS", global = true, value_parser = scripts::value_parser)]
    scripts: Option<scripts::Scripts>,
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

                if let Some(scripts) = &self.scripts {
                    scripts
                        .on_start(name)
                        .wrap_err("failed to run start script after starting")?;
                }
            }
            Command::Break { duration } => {
                store.start_break(Local::now() + Duration::minutes(TryInto::try_into(*duration)?));
                store
                    .write()
                    .wrap_err("could not write state after starting break")?;

                if let Some(scripts) = &self.scripts {
                    scripts
                        .on_break()
                        .wrap_err("failed to run start script after starting")?;
                }
            }
            Command::Stop => {
                store.stop();
                store
                    .write()
                    .wrap_err("could not write state after starting break")?;

                if let Some(scripts) = &self.scripts {
                    scripts
                        .on_stop()
                        .wrap_err("failed to run start script after starting")?;
                }
            }
            Command::Xbar => {
                let now = Local::now();
                match store.state {
                    state::State::NothingIsHappening {} => println!("no task"),
                    state::State::Running { task, until } => {
                        println!("{} ({})", task, Self::humanize_duration(until - now))
                    }
                    state::State::OnBreak { until } => {
                        println!("on break ({})", Self::humanize_duration(until - now))
                    }
                }
            }
            Command::Vex => {
                let store_events = store.watch().wrap_err("could not watch store")?;
                let tick_events = crossbeam_channel::tick(std::time::Duration::from_secs(2));
                let mut rng = rand::thread_rng();

                loop {
                    crossbeam_channel::select! {
                        recv(store_events) -> msg_res => match msg_res {
                            Ok(()) => {
                                store.reload().wrap_err("could not reload store")?;
                            },
                            Err(err) => println!("err: {:#?}", err),
                        },
                        recv(tick_events) -> _msg => {
                            let now = Local::now();

                            let beep_after = match store.state {
                                state::State::NothingIsHappening {} => now,
                                state::State::Running { until, .. } => until,
                                state::State::OnBreak { until } => until,
                            };

                            if now >= beep_after {
                                let what_to_say = THINGS_TO_SAY.choose(&mut rng).unwrap();
                                std::process::Command::new("say").arg(what_to_say).spawn()?;
                            }
                            println!("{}", Self::humanize_duration(beep_after - now));
                        },
                    }
                }
            }
        }

        Ok(())
    }

    fn humanize_duration(duration: chrono::Duration) -> String {
        match duration.num_minutes() {
            0 => format!("{} seconds", duration.num_seconds()),
            1 => format!("1 minute, {} seconds", duration.num_seconds() - 60),
            more => format!("{} minutes", more + 1),
        }
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

    /// Stop permanently (like, for the day or for an extended break)
    Stop,

    /// Show an xbar status message
    Xbar,

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Vex,
}

fn main() -> Result<()> {
    // a builder for `FmtSubscriber`.
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(tracing::Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    color_eyre::install()?;

    Opts::parse().run()?;

    Ok(())
}
