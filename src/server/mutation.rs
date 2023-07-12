use super::session::Session;
use async_graphql::context::Context;
use async_graphql::Object;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct Mutation;

#[Object]
impl Mutation {
    /// Start a new session
    async fn start(
        &self,
        _context: &Context<'_>,
        kind: String,
        #[graphql(desc = "What will you be doing during this session?")] description: String,
        #[graphql(desc = "How long will this session last?")] duration: chrono::Duration,
        #[graphql(desc = "When did this session start? (Omit to start now)")] start_time: Option<
            chrono::DateTime<chrono::Local>,
        >,
    ) -> Result<Session> {
        let start = start_time.unwrap_or_else(|| chrono::Local::now());

        Ok(Session {
            id: 0,
            kind,
            description,
            start_time: start,
            end_time: start + duration,
        })
    }

    /// Extend the current session by a set amount of time
    async fn extend(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "How much time to add?")] _duration: chrono::Duration,
    ) -> Result<Session> {
        color_eyre::eyre::bail!("not implemented yet");
    }

    /// Stop without starting a new session, like for the day or an extended break
    async fn stop(&self, _ctx: &Context<'_>) -> Result<Session> {
        color_eyre::eyre::bail!("not implemented yet");
    }
}
