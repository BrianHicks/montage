use super::error::{Error, Result};
use super::session::Session;
use async_graphql::{Context, Object};
use chrono::Local;

pub struct Query;

#[Object]
impl Query {
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    async fn current_session(&self, context: &Context<'_>) -> Result<Option<Session>> {
        Session::current_session(context.data().map_err(Error::Context)?).await
    }

    async fn sessions(&self, context: &Context<'_>) -> Result<Vec<Session>> {
        Session::for_date(context.data().map_err(Error::Context)?, Local::now()).await
    }
}
