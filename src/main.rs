mod client;
mod server;
mod tokio_spawner;

use crate::tokio_spawner::TokioSpawner;
use async_tungstenite::tungstenite::{
    client::IntoClientRequest, handshake::client::Request, http::HeaderValue,
};
use chrono::{DateTime, Duration, Local};
use clap::Parser;
use color_eyre::eyre::{eyre, Result, WrapErr};
use cynic::http::{CynicReqwestError, ReqwestExt};
use cynic::{GraphQlResponse, MutationBuilder, Operation, QueryBuilder, SubscriptionBuilder};
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use std::str::FromStr;

static THINGS_TO_SAY: [&str; 4] = ["hey", "pick a new task", "Brian", "time for a break?"];

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
            Command::Start {
                description,
                duration,
                client_info,
            } => {
                let query =
                    client::start::StartMutation::build(client::start::StartMutationVariables {
                        description,
                        kind: client::start::Kind::Task,
                        duration: *duration,
                    });

                let session = client_info
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
            Command::Break {
                description,
                duration,
                client_info,
            } => {
                let query =
                    client::start::StartMutation::build(client::start::StartMutationVariables {
                        description,
                        kind: client::start::Kind::Break,
                        duration: *duration,
                    });

                let session = client_info
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
            Command::Watch(client_info) => {
                let query = client::current_session_updates::CurrentSessionUpdates::build(());

                let (connection, _) =
                    async_tungstenite::tokio::connect_async(client_info.request()?)
                        .await
                        .unwrap();

                let (sink, stream) = connection.split();

                let mut client = CynicClientBuilder::new()
                    .build(stream, sink, TokioSpawner::current())
                    .await
                    .unwrap();

                let mut stream = client.streaming_operation(query).await.unwrap();
                while let Some(item) = stream.next().await {
                    println!("{:?}", item);
                }
            }
            Command::Xbar(client_info) => {
                let client = reqwest::Client::new();

                let query = client::current_session::CurrentSessionQuery::build(());

                match client.post(client_info.endpoint()).run_graphql(query).await {
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
            Command::Vex => {
                let mut rng = rand::thread_rng();
                let what_to_say = THINGS_TO_SAY.choose(&mut rng).unwrap();
                todo!("HEY {what_to_say}");
            }
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

static DEFAULT_ADDR: &str = "127.0.0.1";

/// Squatting on a IANA reserved port of a project that I used to work on which got a reserved port
/// but (sadly) never saw real production use. It's super unlikely that I'll ever have a conflict
/// here from a system service since it's reserved!
static DEFAULT_PORT: &str = "4774";

#[derive(Parser, Debug)]
struct GraphQLClientInfo {
    /// The address to bind to
    #[arg(long, default_value = DEFAULT_ADDR, env = "MONTAGE_ADDR")]
    server_addr: std::net::IpAddr,

    /// The port to bind to
    #[arg(long, default_value = DEFAULT_PORT, env = "MONTAGE_PORT")]
    server_port: u16,
}

impl GraphQLClientInfo {
    fn endpoint(&self) -> String {
        format!("http://{}:{}/graphql", self.server_addr, self.server_port)
    }

    async fn make_graphql_request<ResponseData, Vars>(
        &self,
        query: Operation<ResponseData, Vars>,
    ) -> Result<GraphQlResponse<ResponseData>>
    where
        Vars: Serialize,
        ResponseData: DeserializeOwned + 'static,
    {
        let client = reqwest::Client::new();

        client
            .post(self.endpoint())
            .run_graphql(query)
            .await
            .wrap_err("GraphQL request failed")
    }

    fn request(&self) -> Result<Request> {
        let mut request = format!("ws://{}:{}", self.server_addr, self.server_port)
            .into_client_request()
            .wrap_err("could not make a request with addresses provided")?;

        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );

        Ok(request)
    }
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Start a task
    Start {
        /// The name of the task you're working on
        description: String,

        /// How long you're planning to work, in ISO8601 duration format
        #[arg(long)]
        duration: Option<iso8601::Duration>,

        #[command(flatten)]
        client_info: GraphQLClientInfo,
    },

    /// Take a break in between tasks
    Break {
        #[arg(long, default_value = "Break")]
        description: String,

        /// How long you're going to rest, in ISO8601 duration format
        #[arg(long)]
        duration: Option<iso8601::Duration>,

        #[command(flatten)]
        client_info: GraphQLClientInfo,
    },

    Watch(GraphQLClientInfo),

    /// Show an xbar status message
    Xbar(GraphQLClientInfo),

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Vex,

    /// Start the server, which enables the rest of the features!
    Serve {
        /// The address to bind to
        #[arg(long, default_value = DEFAULT_ADDR, env = "MONTAGE_ADDR")]
        addr: std::net::IpAddr,

        /// The port to bind to
        #[arg(long, default_value = DEFAULT_PORT, env = "MONTAGE_PORT")]
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
