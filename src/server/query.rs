use super::error::{Error, Result};
use super::session::Session;
use async_graphql::{Context, Object};
use chrono::{DateTime, Local};

pub struct Query;

#[Object]
impl Query {
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    async fn current_session(&self, context: &Context<'_>) -> Result<Option<Session>> {
        Session::current_session(context.data().map_err(Error::Context)?).await
    }

    async fn sessions_for_date(
        &self,
        context: &Context<'_>,
        when: DateTime<Local>,
    ) -> Result<Vec<Session>> {
        Session::for_date(context.data().map_err(Error::Context)?, when).await
    }
}
