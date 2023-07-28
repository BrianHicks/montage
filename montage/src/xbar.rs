use super::graphql_client::GraphQLClientOptions;
use chrono::Duration;
use color_eyre::eyre::{Result, WrapErr};
use cynic::http::{CynicReqwestError, ReqwestExt};
use cynic::QueryBuilder;

#[derive(Debug, clap::Parser)]
pub struct XBar {
    #[command(flatten)]
    client_options: GraphQLClientOptions,
}

impl XBar {
    pub async fn run(&self) -> Result<()> {
        let http_client = reqwest::Client::new();

        let query = montage_client::current_session::CurrentSessionQuery::build(());

        match http_client
            .post(self.client_options.endpoint())
            .run_graphql(query)
            .await
        {
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

        Ok(())
    }
}
