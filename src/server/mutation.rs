use async_graphql::Object;
use color_eyre::eyre::Result;
use async_graphql::context::Context;

#[derive(Debug)]
pub struct Mutation;

#[Object]
impl Mutation {
    async fn foo(&self, ctx: &Context<'_>) -> Result<bool> {
        tracing::info!("{:#?}", ctx.data::<sqlx::Pool<sqlx::Sqlite>>());
        Ok(true)
    }
}
