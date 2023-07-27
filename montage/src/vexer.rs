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

#[derive(Parser, Debug)]
pub struct Vexer {
    /// How often to say things once the session is over (seconds)
    #[arg(long, default_value = "2")]
    remind_interval: u64,

    #[command(flatten)]
    client: crate::graphql_client::GraphQLClient,
}

impl Vexer {
    pub async fn run(&self) -> Result<()> {
        let mut state = State::default();
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(self.remind_interval));

        let query = CurrentSessionUpdates::build(());
        let (connection, _) = async_tungstenite::tokio::connect_async(self.client.request()?)
            .await
            .wrap_err_with(|| format!("could not connect to `{}`", self.client.ws_endpoint()))?;

        let (sink, stream) = connection.split();
        let mut client = CynicClientBuilder::new()
            .build(stream, sink, TokioSpawner::current())
            .await
            .wrap_err("could not construct a Cynic client")?;

        let mut sessions_stream = client
            .streaming_operation(query)
            .await
            .wrap_err("could not start streaming")?;

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
                            state.got_new_session(session_opt);
                        },
                        Some(Err(err)) => {
                            tracing::error!(err=?err, "error getting next sesson");
                        }
                        None => break,
                    }
                },
                _ = interval.tick() => state.tick()?,
            }
        }

        // TODO: disconnect properly

        Ok(())
    }
}

#[derive(Debug)]
struct State {
    session: Option<montage_client::current_session_updates::Session>,
    rng: ThreadRng,
}

impl State {
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

impl Default for State {
    fn default() -> Self {
        Self {
            session: None,
            rng: rand::thread_rng(),
        }
    }
}
