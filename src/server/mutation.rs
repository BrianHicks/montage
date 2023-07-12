use async_graphql::Object;
use color_eyre::eyre::Result;
use async_graphql::context::Context;
use super::session::Session;

#[derive(Debug)]
pub struct Mutation;

#[Object]
impl Mutation {
    async fn start(&self, ctx: &Context<'_>) -> Result<Session> {
        Ok(Session {
            id: 0,
            kind: String::from("hey"),
            description: String::from("desc"),
            start_time: chrono::Local::now(),
            end_time: chrono::Local::now(),
        })
    }

    async fn foo(&self, ctx: &Context<'_>) -> Result<bool> {
        tracing::info!("{:#?}", ctx.data::<sqlx::Pool<sqlx::Sqlite>>());
        Ok(true)
    }
}
