use super::scripts::Script;
use super::TokioSpawner;
use chrono::{DateTime, Local};
use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use cynic::SubscriptionBuilder;
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use montage_client::current_session_updates::Session;
use montage_client::current_session_updates::{CurrentSessionUpdates, Kind};
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::select;

static MAXIMUM_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Parser, Debug)]
pub struct VexerConfig {
    /// How often to say things once the session is over (seconds)
    #[arg(long, default_value = "2")]
    remind_interval: u64,

    /// What command to run for TTS
    #[arg(long, default_value = "say")]
    tts_command: String,

    /// Args to the TTS command
    #[arg(long)]
    tts_arg: Vec<String>,

    /// Get reminders at these intervals before the end of the session (in minutes)
    #[arg(long, short, default_values = ["15", "10", "5", "1"])]
    reminder_at: Vec<i64>,

    #[arg(long)]
    script_dir: Option<PathBuf>,

    /// The computer will say your name sometimes. What is it?
    #[arg(long, default_value = "Brian")]
    your_name: String,

    /// How long should you work before the computer prompts you to take a break at the end of a
    /// session? (minutes)
    #[arg(long, default_value = "25")]
    ideal_work_session_length: i64,

    #[command(flatten)]
    client: crate::graphql_client::GraphQLClientOptions,
}

impl VexerConfig {
    pub async fn run(&self) -> Result<()> {
        let mut vexer = Vexer::new(self);
        vexer.run().await
    }
}

#[derive(Debug)]
struct Vexer<'config> {
    config: &'config VexerConfig,
    session: Option<Session>,
    rng: ThreadRng,
    backoff: std::time::Duration,

    reminders_to_give: HashSet<chrono::Duration>,
    reminders_given: HashSet<chrono::Duration>,

    sent_session_ended: bool,
    current_work_session_started: Option<DateTime<Local>>,
}

impl<'config> Vexer<'config> {
    fn new(config: &'config VexerConfig) -> Self {
        Self {
            config,
            session: None,
            rng: rand::thread_rng(),
            backoff: std::time::Duration::from_secs(0),

            reminders_to_give: config
                .reminder_at
                .iter()
                .map(|minutes| chrono::Duration::minutes(*minutes))
                .collect(),
            reminders_given: HashSet::with_capacity(config.reminder_at.len()),
            sent_session_ended: false,
            current_work_session_started: None,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.remind_interval,
        ));

        loop {
            if !self.backoff.is_zero() {
                tracing::warn!(
                    seconds = self.backoff.as_secs(),
                    "encountered errors; backing off"
                );
            }
            tokio::time::sleep(self.backoff).await;

            let (connection, _) = match async_tungstenite::tokio::connect_async(
                self.config.client.request()?,
            )
            .await
            {
                Ok(conn) => conn,
                Err(err) => {
                    tracing::error!(err = err.to_string(), "could not connect");
                    self.increment_backoff();
                    continue;
                }
            };

            let (sink, stream) = connection.split();
            let mut client = CynicClientBuilder::new()
                .build(stream, sink, TokioSpawner::current())
                .await
                .wrap_err("could not construct a Cynic client")?;

            let query = CurrentSessionUpdates::build(());
            let mut sessions_stream = client
                .streaming_operation(query)
                .await
                .wrap_err("could not start streaming")?;

            self.successfully_connected();

            loop {
                select! {
                    new_session_resp_opt = sessions_stream.next() => {
                        match new_session_resp_opt {
                            Some(Ok(new_session_resp)) => {
                                let session_opt = new_session_resp.data.and_then(|r| r.current_session);
                                tracing::info!(
                                    session=?session_opt,
                                    "got a new session"
                                );
                                if let Err(err) = self.got_new_session(session_opt).await {
                                    tracing::error!(err=?err, "error in new session");
                                };
                            },
                            Some(Err(err)) => {
                                tracing::error!(err=?err, "error getting next session");
                            }
                            None => {
                                tracing::info!("disconnected from websocket stream, trying to reconnect");
                                break
                            },
                        }
                    },
                    _ = interval.tick() => if let Err(err) = self.tick().await {
                        tracing::error!(err=?err, "error in time tick");
                    },
                }
            }
        }
    }

    fn increment_backoff(&mut self) {
        if self.backoff.is_zero() {
            self.backoff = std::time::Duration::from_secs(1);
        } else {
            self.backoff *= 2;
        }

        self.backoff = std::cmp::min(self.backoff, MAXIMUM_BACKOFF);

        tracing::debug!(backoff = ?self.backoff, "increasing backoff");
    }

    fn successfully_connected(&mut self) {
        tracing::info!("successfully connected");
        self.backoff = std::time::Duration::from_secs(0);
    }

    async fn got_new_session(
        &mut self,
        session_opt: Option<montage_client::current_session_updates::Session>,
    ) -> Result<()> {
        let mut same_session = false;

        if let Some(old_session) = &self.session {
            if let Some(session) = &session_opt {
                if session.id == old_session.id {
                    same_session = true;
                } else {
                    self.run_script(Script::SessionEnded {
                        session: old_session,
                        next_session: session,
                    })
                    .await?;
                }
            }
        }

        self.session = session_opt;
        self.sent_session_ended = false;

        if let Some(session) = &self.session {
            self.run_script(if same_session {
                Script::SessionExtended { session }
            } else {
                Script::NewSession { session }
            })
            .await?;

            match session.kind {
                Kind::Task | Kind::Meeting => {
                    if self.current_work_session_started.is_none() {
                        self.current_work_session_started = Some(Local::now());
                    }
                }

                Kind::Break | Kind::Offline => self.current_work_session_started = None,
            };

            let time_remaining = session.projected_end_time - Local::now();

            self.reminders_given.clear();
            self.reminders_to_give.iter().for_each(|reminder| {
                if reminder >= &time_remaining {
                    self.reminders_given.insert(*reminder);
                }
            });
            tracing::info!(reminders=?self.reminders_to_give.difference(&self.reminders_given), "reset reminders");
        }

        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let time_remaining = session.projected_end_time - Local::now();

            for reminder in &self.reminders_to_give {
                // could use difference but it results in an immutable borrow and we need it to be
                // immutable just below.
                if self.reminders_given.contains(&reminder) {
                    continue;
                }

                if reminder >= &time_remaining {
                    futures::try_join!(
                        self.give_reminder(&reminder),
                        self.run_script(Script::Reminder { session, reminder }),
                    )?;
                    self.reminders_given.insert(*reminder);
                }
            }

            if time_remaining < chrono::Duration::zero() {
                tracing::info!(?time_remaining, "over time");

                // these can't be run in parallel because `annoy` runs in parallel. Oh well!
                self.run_script(Script::SessionOverTime { session }).await?;
                self.annoy().await?;
            }
        }

        Ok(())
    }

    async fn give_reminder(&self, reminder_at: &chrono::Duration) -> Result<()> {
        let minutes = reminder_at.num_minutes();

        if minutes == 1 {
            self.say("one minute left").await
        } else {
            self.say(&format!("{minutes} minutes left")).await
        }
    }

    async fn annoy(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let options = match session.kind {
                Kind::Task => {
                    let mut options =
                        vec![String::from("hey"), String::from("need another minute?")];
                    options.push(self.config.your_name.clone());

                    if self.working_over_ideal_work_session_length() {
                        match self.current_work_session_length() {
                            Some(length) => options.push(format!(
                                "You've been working for {} minutes.",
                                length.num_minutes(),
                            )),

                            None => tracing::warn!("current_work_session_length was None, despite working_over_ideal_work_session_length being true"),
                        }
                    }

                    options
                }

                Kind::Break | Kind::Offline => {
                    let mut options = vec![
                        String::from("hey"),
                        String::from("pick a new task"),
                        String::from("ready to go?"),
                        String::from("let's do this!"),
                    ];
                    options.push(self.config.your_name.clone());

                    options
                }

                // We don't annoy when meetings end because sometimes they run long and it's
                // awkward to have the computer start saying silly things on Zoom!
                Kind::Meeting => vec![],
            };

            if let Some(what_to_say) = options.choose(&mut self.rng) {
                self.say(what_to_say).await?
            }
        }

        Ok(())
    }

    async fn say(&self, what_to_say: &str) -> Result<()> {
        if self.in_meeting() {
            return Ok(());
        }

        let status = Command::new(&self.config.tts_command)
            .args(&self.config.tts_arg)
            .arg(what_to_say)
            .status()
            .await
            .wrap_err("failed to run `say`")?;

        if !status.success() {
            bail!("`say` failed with status {}", status)
        }

        Ok(())
    }

    fn in_meeting(&self) -> bool {
        match self.session {
            Some(Session { kind, .. }) => kind == Kind::Meeting,
            _ => false,
        }
    }

    fn current_work_session_length(&self) -> Option<chrono::Duration> {
        self.current_work_session_started
            .map(|start| Local::now() - start)
    }

    fn working_over_ideal_work_session_length(&self) -> bool {
        self.current_work_session_length()
            .map(|len| len.num_minutes() > self.config.ideal_work_session_length)
            .unwrap_or(false)
    }

    async fn run_script(&self, script: Script<'_>) -> Result<()> {
        script.run_from(&self.config.script_dir).await
    }
}
