use crate::client::current_session_updates::CurrentSessionUpdates;
use crate::TokioSpawner;
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use cynic::SubscriptionBuilder;
use futures::StreamExt;
use graphql_ws_client::CynicClientBuilder;
use rand::seq::SliceRandom;
use tokio::select;

static THINGS_TO_SAY: [&str; 4] = ["hey", "pick a new task", "Brian", "time for a break?"];

#[derive(Parser, Debug)]
pub struct Vexer {
    #[command(flatten)]
    client: crate::graphql_client::GraphQLClient,
}

impl Vexer {
    pub async fn run(&self) -> Result<()> {
        let mut session = None;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

        let mut rng = rand::thread_rng();
        let what_to_say = THINGS_TO_SAY.choose(&mut rng).unwrap();
        println!("HEY {what_to_say}");

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
                new_session_opt = sessions_stream.next() => {
                    match new_session_opt {
                        Some(new_session) =>{
                            session = Some(new_session);
                            println!("{session:#?}");
                        },
                        None => break,
                    }
                },
                _ = interval.tick() => println!("tick! {session:?}"),
            }
        }

        Ok(())
    }
}
