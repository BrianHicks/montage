use super::error::{Error, Result};
use super::session::Session;
use async_graphql::{Context, Object};

pub struct Query;

#[Object]
impl Query {
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    async fn current_session(&self, context: &Context<'_>) -> Result<Option<Session>> {
        Session::current_session(context.data().map_err(Error::ContextError)?).await
    }
}
