use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Opts {
    #[command(subcommand)]
    command: Command,
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

    /// Show the current task and remaining time OR that there's no task
    Status,

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Daemon,
}

fn main() {
    let opts = Opts::parse();

    println!("{:#?}", opts);
}
