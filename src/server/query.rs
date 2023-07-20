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

    /// Get a report on the sessions in a given range (start and end will be treated as a date
    /// range inclusive of sessions on both the start and end days. To get just a single day, pass
    /// the same day twice.)
    async fn report(
        &self,
        context: &Context<'_>,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<Session>> {
        Session::for_range_inclusive(context.data().map_err(Error::Context)?, start, end).await
    }
}
