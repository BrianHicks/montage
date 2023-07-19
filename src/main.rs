mod client;
mod graphql_client;
mod server;
mod tokio_spawner;
mod vexer;

use crate::graphql_client::GraphQLClient;
use crate::tokio_spawner::TokioSpawner;
use chrono::{DateTime, Duration, Local};
use clap::Parser;
use client::current_session_updates::CurrentSessionUpdates;
use color_eyre::eyre::{eyre, Result, WrapErr};
use cynic::http::{CynicReqwestError, ReqwestExt};
use cynic::{MutationBuilder, QueryBuilder, SubscriptionBuilder};
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Opts {
    #[command(subcommand)]
    command: Command,

    #[arg(long, env = "MONTAGE_LOG_LEVEL", global = true, default_value = "INFO")]
    log_level: tracing::Level,

    #[arg(long, env = "MONTAGE_DB", global = true)]
    db_dir: Option<PathBuf>,
}

impl Opts {
    async fn run(&self) -> Result<()> {
        match &self.command {
            Command::Start(StartOrBreak {
                description,
                duration,
                client,
            }) => {
                let query =
                    client::start::StartMutation::build(client::start::StartMutationVariables {
                        description,
                        kind: client::start::Kind::Task,
                        duration: duration
                            .map(|d| iso8601::duration(&format!("PT{}M", d)).unwrap()),
                    });

                let session = client
                    .make_graphql_request(query)
                    .await?
                    .data
                    .expect("a non-null session")
                    .start;

                println!(
                    "Started \"{}\", running for {} minutes until {}",
                    session.description,
                    Self::humanize_duration_minutes(session.duration)?,
                    Self::humanize_time_12hr(session.projected_end_time),
                )
            }
            Command::Break(StartOrBreak {
                description,
                duration,
                client,
            }) => {
                let query =
                    client::start::StartMutation::build(client::start::StartMutationVariables {
                        description,
                        kind: client::start::Kind::Break,
                        duration: duration
                            .map(|d| iso8601::duration(&format!("PT{}M", d)).unwrap()),
                    });

                let session = client
                    .make_graphql_request(query)
                    .await?
                    .data
                    .expect("a non-null session")
                    .start;

                println!(
                    "Started break, running for {} minutes until {}",
                    Self::humanize_duration_minutes(session.duration)?,
                    Self::humanize_time_12hr(session.projected_end_time),
                )
            }
            Command::Extend { by, to, client } => {
                if let Some(duration) = by {
                    let query = client::extend_by::ExtendByMutation::build(
                        client::extend_by::ExtendByMutationVariables {
                            duration: *duration,
                        },
                    );

                    let session = client
                        .make_graphql_request(query)
                        .await?
                        .data
                        .expect("a non-null session")
                        .extend_by;

                    println!(
                        "{} extended by {} minutes to end at {}",
                        session.description,
                        Self::humanize_duration_minutes(*duration)?,
                        Self::humanize_time_12hr(session.projected_end_time),
                    );
                } else if let Some(target) = to {
                    let query = client::extend_to::ExtendToMutation::build(
                        client::extend_to::ExtendToMutationVariables { target: *target },
                    );

                    let session = client
                        .make_graphql_request(query)
                        .await?
                        .data
                        .expect("a non-null session")
                        .extend_to;

                    println!(
                        "{} extended to end at {}",
                        session.description,
                        Self::humanize_time_12hr(session.projected_end_time),
                    );
                } else {
                    color_eyre::eyre::bail!("got neither --by nor --to. This should not happen!");
                };
            }
            Command::Watch(client) => {
                let query = CurrentSessionUpdates::build(());
                let (connection, _) = async_tungstenite::tokio::connect_async(client.request()?)
                    .await
                    .unwrap();

                let (sink, stream) = connection.split();
                let mut client = CynicClientBuilder::new()
                    .build(stream, sink, TokioSpawner::current())
                    .await
                    .unwrap();

                let mut sessions_stream = client
                    .streaming_operation(query)
                    .await
                    .wrap_err("could not start streaming")?;

                while let Some(item) = sessions_stream.next().await {
                    println!("{:?}", item);
                }

                // TODO: gracefully drop the connection
            }
            Command::Xbar(client) => {
                let http_client = reqwest::Client::new();

                let query = client::current_session::CurrentSessionQuery::build(());

                match http_client.post(client.endpoint()).run_graphql(query).await {
                    Err(CynicReqwestError::ReqwestError(err)) if err.is_connect() => {
                        // a message for the xbar status line
                        eprintln!("⚠️ failed to connect to server");

                        // a message to expand on
                        return Err(err).wrap_err("GraphQL request failed");
                    }
                    Err(err) => return Err(err).wrap_err("GraphQL request failed"),
                    Ok(resp) => {
                        let session = resp
                            .data
                            .expect("a non-null session")
                            .current_session
                            .expect("a current session");

                        let duration = Duration::from_std(std::time::Duration::from(
                            session.remaining_time.expect("remaining time"),
                        ))
                        .wrap_err("could not parse duration")?;
                        let minutes = duration.num_minutes();

                        println!(
                            "⏰ {} ({}:{:02})",
                            session.description,
                            minutes,
                            duration.num_seconds() - minutes * 60,
                        );
                    }
                };
            }
            Command::Vex(vexer) => vexer.run().await?,
            Command::Serve { addr, port } => {
                server::serve(self.open_sqlite_database().await?, *addr, *port).await?
            }
            Command::ShowGraphqlSchema => {
                println!(
                    "{}",
                    server::schema(self.open_sqlite_database().await?)
                        .await?
                        .sdl()
                )
            }
        }

        Ok(())
    }

    fn humanize_time_12hr(time: DateTime<Local>) -> String {
        if Local::now().date_naive() == time.date_naive() {
            time.format("%I:%M %P").to_string()
        } else {
            time.format("%Y-%m-%d %I:%M %P").to_string()
        }
    }

    fn humanize_duration_minutes(duration: iso8601::Duration) -> Result<i64> {
        Ok(Duration::from_std(std::time::Duration::from(duration))
            .wrap_err("could not parse duration")?
            .num_minutes())
    }

    async fn open_sqlite_database(&self) -> Result<Pool<Sqlite>> {
        // TODO: could we get rid of the to_owned calls somehow?
        let db_dir = match &self.db_dir {
            Some(db) => db
                .parent()
                .map(|parent| parent.to_owned())
                .unwrap_or_else(|| PathBuf::from(".")),
            None => directories::ProjectDirs::from("zone", "bytes", "montage")
                .ok_or(eyre!("could not determine config location"))?
                .data_local_dir()
                .to_owned(),
        };

        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).wrap_err("could not create database directory")?;
        }

        let db_path = format!("sqlite://{}", db_dir.join("montage.sqlite3").display());

        let connection_options = SqliteConnectOptions::from_str(&db_path)?.create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(connection_options)
            .await
            .wrap_err_with(|| {
                format!("could not make connection to sqlite database at `{db_path}`",)
            })?;

        sqlx::migrate!("db/migrations")
            .run(&pool)
            .await
            .wrap_err("could not run migrations")?;

        Ok(pool)
    }
}

#[derive(Parser, Debug)]
struct StartOrBreak {
    /// The name of the task you're working on
    description: String,

    /// How long you're planning to work, in minutes
    #[arg(long)]
    duration: Option<usize>,

    #[command(flatten)]
    client: GraphQLClient,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Start a task
    Start(StartOrBreak),

    /// Take a break in between tasks
    Break(StartOrBreak),

    /// Add some more time onto the current session
    Extend {
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        by: Option<iso8601::Duration>,

        #[arg(long, conflicts_with = "by", required_unless_present = "by")]
        to: Option<DateTime<Local>>,

        #[command(flatten)]
        client: GraphQLClient,
    },

    Watch(GraphQLClient),

    /// Show an xbar status message
    Xbar(GraphQLClient),

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Vex(crate::vexer::Vexer),

    /// Start the server, which enables the rest of the features!
    Serve {
        /// The address to bind to
        #[arg(long, default_value = crate::graphql_client::DEFAULT_ADDR, env = "MONTAGE_ADDR")]
        addr: std::net::IpAddr,

        /// The port to bind to
        #[arg(long, default_value = crate::graphql_client::DEFAULT_PORT, env = "MONTAGE_PORT")]
        port: u16,
    },

    /// Export the GraphQL SDL for the server
    ShowGraphqlSchema,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opts = Opts::parse();

    // a builder for `FmtSubscriber`.
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(opts.log_level)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    opts.run().await
}
