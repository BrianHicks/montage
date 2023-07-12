use super::error::{Error, Result};
use super::kind::Kind;
use super::session::Session;
use async_graphql::context::Context;
use async_graphql::Object;

#[derive(Debug)]
pub struct Mutation;

#[Object]
impl Mutation {
    /// Start a new session
    async fn start(
        &self,
        context: &Context<'_>,
        #[graphql(desc = "What kind of session will this be?")] kind: Kind,
        #[graphql(desc = "What will you be doing during this session?")] description: String,
        #[graphql(
            desc = "How long will this session last? (If omitted, we'll decide based on the session type)"
        )]
        duration: Option<chrono::Duration>,
        #[graphql(desc = "When did this session start? (Omit to start now)")] start_time: Option<
            chrono::DateTime<chrono::Local>,
        >,
    ) -> Result<Session> {
        let final_start = start_time.unwrap_or_else(chrono::Local::now);

        let final_duration = duration.unwrap_or_else(|| kind.default_session_length());

        Session::start(
            context.data().map_err(Error::ContextError)?,
            kind,
            &description,
            final_start,
            final_duration,
        )
        .await
    }

    /// Extend the current session by a set amount of time
    async fn extend(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "How much time to add?")] _duration: chrono::Duration,
    ) -> Result<Session> {
        Err(Error::NotImplemented)
    }
}
