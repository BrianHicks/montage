mod graphql_client;
mod scripts;
mod tokio_spawner;
mod vexer;
mod xbar;

use crate::graphql_client::GraphQLClientOptions;
use crate::tokio_spawner::TokioSpawner;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone};
use clap::Parser;
use color_eyre::eyre::{bail, eyre, Result, WrapErr};

use cynic::{MutationBuilder, QueryBuilder, SubscriptionBuilder};
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use handlebars::{handlebars_helper, Handlebars};
use montage_client::current_session_updates::CurrentSessionUpdates;
use montage_client::report::Report;
use serde::Serialize;
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
}

impl Opts {
    async fn run(&self) -> Result<()> {
        match &self.command {
            Command::Start {
                description,
                duration,
                until,
                client,
                is_meeting,
                is_break,
                is_offline,
            } => {
                let query = montage_client::start::StartMutation::build(
                    montage_client::start::StartMutationVariables {
                        description,
                        kind: if *is_meeting {
                            montage_client::start::Kind::Meeting
                        } else if *is_break {
                            montage_client::start::Kind::Break
                        } else if *is_offline {
                            montage_client::start::Kind::Offline
                        } else {
                            montage_client::start::Kind::Task
                        },
                        duration: Self::duration_from_options(duration, until)?,
                    },
                );

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
            Command::Extend { by, to, client } => {
                if let Some(duration) = by {
                    let query = montage_client::extend_by::ExtendByMutation::build(
                        montage_client::extend_by::ExtendByMutationVariables {
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
                    let query = montage_client::extend_to::ExtendToMutation::build(
                        montage_client::extend_to::ExtendToMutationVariables { target: *target },
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
                    bail!("got neither --by nor --to. This should not happen!");
                };
            }
            Command::Report {
                from: naive_from,
                to: naive_to,
                no_log,
                no_task_totals,
                include_long_breaks_in_summary,
                template,
                client,
            } => {
                let from = naive_from
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .and_then(|date| date.and_local_timezone(Local).into())
                    .unwrap_or_else(|| {
                        let now = Local::now();

                        now.timezone()
                            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                    })
                    .unwrap();

                let to = naive_to
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|date| date.and_local_timezone(Local).unwrap())
                    .unwrap_or(from);

                let query = montage_client::report::ReportQuery::build(
                    montage_client::report::ReportQueryVariables {
                        start: from,
                        end: to,
                    },
                );

                let report = client
                    .make_graphql_request(query)
                    .await?
                    .data
                    .ok_or(eyre!("data was null"))?
                    .report;

                let date_format = "%A, %B %d";
                let date_range = if report.start == report.end {
                    format!("on {}", report.start.format(date_format))
                } else {
                    format!(
                        "from {} to {}",
                        report.start.format(date_format),
                        report.end.format(date_format)
                    )
                };

                #[derive(Serialize)]
                struct Context {
                    report: Report,
                    date_range: String,
                    include_sessions: bool,
                    include_task_totals: bool,
                    include_long_breaks_in_summary: bool,
                }

                let context = Context {
                    report,
                    date_range,
                    include_sessions: !no_log,
                    include_task_totals: !no_task_totals,
                    include_long_breaks_in_summary: *include_long_breaks_in_summary,
                };

                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);

                handlebars_helper!(
                    hms: |duration_str: String| {
                        // TODO: make this less panicky
                        let duration = Duration::from_std(std::time::Duration::from(
                            iso8601::duration(&duration_str).expect("a valid ISO8601 duration string"),
                        )).expect("duration to not be out of bounds");

                        if duration.num_seconds() < 60 {
                            format!("{}s", duration.num_seconds())
                        } else if duration.num_minutes() < 60 {
                            let minutes = duration.num_minutes();

                            format!(
                                "{}m {}s",
                                minutes,
                                duration.num_seconds() - minutes * 60,
                            )
                        } else {
                            let hours = duration.num_hours();
                            let minutes = duration.num_minutes();

                            format!(
                                "{}h {}m {}s",
                                hours,
                                minutes - hours * 60,
                                duration.num_seconds() - minutes * 60,
                            )
                        }
                    }
                );
                handlebars.register_helper("hms", Box::new(hms));

                handlebars_helper!(
                    time: |when: DateTime<Local>| {
                        when.format("%-l:%M %P").to_string()
                    }
                );
                handlebars.register_helper("time", Box::new(time));

                handlebars_helper!(lower: |input: String| input.to_ascii_lowercase());
                handlebars.register_helper("lower", Box::new(lower));

                let default_template = String::from("## Montage Sessions\n\n{{> date_range}}\n\n\n{{> totals report.totals}}\n\n{{#if include_task_totals}}\n\n### Task Totals\n\n{{#each report.totals.sessions_by_description}}- {{>task_by_description}}\n{{/each}}{{/if}}{{#if include_sessions}}\n### Log\n\n{{#each report.sessions}}- {{>session}}\n{{/each}}{{/if}}");

                handlebars.register_template_string::<String>(
                    "report",
                    match &template {
                        Some(t) => t.to_string(),
                        None => default_template,
                    },
                )?;

                handlebars.register_template_string(
                    "date_range",
                    "{{len report.sessions}} sessions {{date_range}}",
                )?;

                handlebars.register_template_string(
                    "totals",
                    "**{{hms task}}** spent on tasks, **{{hms meeting}}** in meetings, and **{{hms short_break}}** on short breaks for a total of **{{hms working}}**{{#if include_long_breaks_in_summary}} In addition, you tracked **{{hms long_break}}** on long breaks{{/if}}."
                )?;

                handlebars.register_template_string(
                    "session",
                    "**{{time start_time}}** {{description}} ({{lower kind}}, {{hms actual_duration}})",
                )?;

                handlebars.register_template_string(
                    "task_by_description",
                    "**{{hms total}}** {{description}} ({{lower kind}})",
                )?;

                println!("{}", handlebars.render("report", &context)?);
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
            Command::Xbar(xbar) => xbar.run().await?,
            Command::Vex(vexer) => vexer.run().await?,
            Command::Serve { addr, port, db_dir } => {
                montage_server::serve(
                    self.open_sqlite_database(db_dir.clone()).await?,
                    *addr,
                    *port,
                )
                .await?
            }
            Command::ShowGraphqlSchema => {
                println!(
                    "{}",
                    montage_server::schema(self.open_sqlite_database(None).await?)
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

    async fn open_sqlite_database(&self, db_dir_option: Option<PathBuf>) -> Result<Pool<Sqlite>> {
        // TODO: could we get rid of the to_owned calls somehow?
        let db_dir = match db_dir_option {
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

        Ok(pool)
    }

    fn duration_from_options(
        duration: &Option<usize>,
        until: &Option<DateTime<Local>>,
    ) -> Result<Option<iso8601::Duration>> {
        match (duration, until) {
            (Some(minutes), None) => {
                let duration_from_minutes = iso8601::duration(&format!("PT{}M", minutes))
                    .expect("configuration error: could not parse that amount of minutes");
                Ok(Some(duration_from_minutes))
            }
            (None, Some(time)) => {
                let now = Local::now();
                let duration = if now > *time {
                    now - *time
                } else {
                    *time - now
                };

                Ok(Some(iso8601::duration(&duration.to_string()).unwrap()))
            }
            (Some(_), Some(_)) => {
                bail!("got both --duration and --until. Configuration error in montage!")
            }
            (None, None) => Ok(None),
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Start a task
    Start {
        /// The task you'll be doing
        description: String,

        /// The length of the task, in minutes
        #[arg(long, conflicts_with = "until")]
        duration: Option<usize>,

        /// Work on a task until a specific time
        #[arg(long, conflicts_with = "duration")]
        until: Option<DateTime<Local>>,

        /// Is this session a meeting?
        #[arg(long("meeting"), conflicts_with_all = ["is_break", "is_offline"])]
        is_meeting: bool,

        /// Is this session a break?
        #[arg(long("break"), conflicts_with_all = ["is_meeting", "is_offline"])]
        is_break: bool,

        /// Is this session offline time?
        #[arg(long("offline"), conflicts_with_all = ["is_break", "is_meeting"])]
        is_offline: bool,

        #[command(flatten)]
        client: GraphQLClientOptions,
    },

    /// Add some more time onto the current session
    Extend {
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        by: Option<iso8601::Duration>,

        #[arg(long, conflicts_with = "by", required_unless_present = "by")]
        to: Option<DateTime<Local>>,

        #[command(flatten)]
        client: GraphQLClientOptions,
    },

    /// Report on the sessions specified in the current days (inclusive).
    Report {
        /// The starting date. If omitted, uses today's date. Assumed to be in the local time zone.
        from: Option<NaiveDate>,

        /// The ending date. If omitted, you'll just get a report for the starting date. Assumed
        /// to be in the local time zone.
        to: Option<NaiveDate>,

        /// If passed, don't include the list of sessions by time.
        #[clap(long)]
        no_log: bool,

        /// If passed, don't include the total time on each task.
        #[clap(long)]
        no_task_totals: bool,

        /// Include long breaks. These are typically just times to get the vexer off my back
        /// (lunch, overnight, etc.) so they aren't that interesting to include in a report.
        #[clap(long)]
        include_long_breaks_in_summary: bool,

        /// The Handlebars template to use for rendering the report.
        ///
        /// There are helpers and sub-templates available, but you'll have to look through the
        /// program source to get them for now!
        #[clap(long)]
        template: Option<String>,

        #[command(flatten)]
        client: GraphQLClientOptions,
    },

    Watch(GraphQLClientOptions),

    /// Show an xbar status message
    Xbar(xbar::XBar),

    /// Run background tasks, like being annoying when there's not an active task or break
    /// running.
    Vex(crate::vexer::VexerConfig),

    /// Start the server, which enables the rest of the features!
    Serve {
        /// The address to bind to
        #[arg(long, default_value = crate::graphql_client::DEFAULT_ADDR, env = "MONTAGE_ADDR")]
        addr: std::net::IpAddr,

        /// The port to bind to
        #[arg(long, default_value = crate::graphql_client::DEFAULT_PORT, env = "MONTAGE_PORT")]
        port: u16,

        /// Where to store the database
        #[arg(long, env = "MONTAGE_DB")]
        db_dir: Option<PathBuf>,
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
