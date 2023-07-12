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
            kind: kind,
            description: description,
            start_time: start,
            end_time: start + duration,
        })
    }

    async fn foo(&self, ctx: &Context<'_>) -> Result<bool> {
        tracing::info!("{:#?}", ctx.data::<sqlx::Pool<sqlx::Sqlite>>());
        Ok(true)
    }
}
