use super::graphql_client::GraphQLClientOptions;
use chrono::Duration;
use color_eyre::eyre::{Result, WrapErr};
use cynic::http::{CynicReqwestError, ReqwestExt};
use cynic::QueryBuilder;
use montage_client::current_session::{Kind, Session};

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

                println!("{}", Self::format(&session)?);
            }
        };

        Ok(())
    }

    fn format(session: &Session) -> Result<String> {
        let duration = Duration::from_std(std::time::Duration::from(
            session.remaining_time.expect("remaining time"),
        ))
        .wrap_err("could not parse duration")?;
        let minutes = duration.num_minutes();

        Ok(format!(
            "{} {} ({}:{:02})",
            Self::emoji(&session),
            Self::escape(&session.description),
            minutes,
            duration.num_seconds() - minutes * 60,
        ))
    }

    fn escape(unescaped: &str) -> String {
        unescaped.replace('|', "\\|")
    }

    fn emoji(session: &Session) -> String {
        match session.kind {
            Kind::Task => "⏰",
            Kind::Break => "☕️",
            Kind::Meeting => "🗣",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use montage_client::current_session::Kind;

    #[test]
    fn format_escapes_bars_in_description() {
        let session = Session {
            description: String::from("A | B | C"),

            // none of the rest of these are coherent. Don't worry about it.
            duration: iso8601::duration("PT5M").unwrap(),
            end_time: None,
            kind: Kind::Task,
            projected_end_time: Local::now(),
            remaining_time: Some(iso8601::duration("PT5M").unwrap()),
            start_time: Local::now(),
        };

        let formatted = XBar::format(&session).unwrap();

        assert_eq!(formatted.lines().next().unwrap(), "⏰ A \\| B \\| C (5:00)")
    }
}
