use super::TokioSpawner;
use chrono::Local;
use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use cynic::SubscriptionBuilder;
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use montage_client::current_session_updates::CurrentSessionUpdates;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::process::Command;
use tokio::select;

static THINGS_TO_SAY: [&str; 4] = ["hey", "pick a new task", "Brian", "time for a break?"];

static MAXIMUM_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Parser, Debug)]
pub struct VexerConfig {
    /// How often to say things once the session is over (seconds)
    #[arg(long, default_value = "2")]
    remind_interval: u64,

    #[command(flatten)]
    client: crate::graphql_client::GraphQLClientOptions,
}

impl VexerConfig {
    pub async fn run(&self) -> Result<()> {
        let mut vexer = Vexer::default();
        vexer.run(self).await
    }
}

#[derive(Debug)]
struct Vexer {
    session: Option<montage_client::current_session_updates::Session>,
    rng: ThreadRng,
    backoff: std::time::Duration,
}

impl Vexer {
    async fn run(&mut self, config: &VexerConfig) -> Result<()> {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(config.remind_interval));

        loop {
            if !self.backoff.is_zero() {
                tracing::warn!(
                    seconds = self.backoff.as_secs(),
                    "encountered errors; backing off"
                );
            }
            tokio::time::sleep(self.backoff).await;

            let (connection, _) =
                match async_tungstenite::tokio::connect_async(config.client.request()?).await {
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
                                self.got_new_session(session_opt);
                            },
                            Some(Err(err)) => {
                                tracing::error!(err=?err, "error getting next sesson");
                            }
                            None => {
                                tracing::info!("disconnected from websocket stream, trying to reconnect");
                                break
                            },
                        }
                    },
                    _ = interval.tick() => if let Err(err) = self.tick() {
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

    fn got_new_session(
        &mut self,
        session_opt: Option<montage_client::current_session_updates::Session>,
    ) {
        self.session = session_opt
    }

    fn tick(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let time_remaining = session.projected_end_time - Local::now();

            if time_remaining < chrono::Duration::zero() {
                tracing::info!(?time_remaining, "over time");

                if !session.is_meeting() {
                    let what_to_say = THINGS_TO_SAY
                        .choose(&mut self.rng)
                        .expect("THINGS_TO_SAY should always have at least one item");

                    let status = Command::new("say")
                        .arg(what_to_say)
                        .status()
                        .wrap_err("failed to run `say`")?;

                    if !status.success() {
                        bail!("`say` failed with status {}", status)
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Vexer {
    fn default() -> Self {
        Self {
            session: None,
            rng: rand::thread_rng(),
            backoff: std::time::Duration::from_secs(0),
        }
    }
}
