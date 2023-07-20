use super::error::Result;
use crate::server::Session;
use async_graphql::SimpleObject;
use chrono::{DateTime, Local};
use sqlx::{Pool, Sqlite};

/// A report on sessons started and ended during a given day plus some statistics.
#[derive(SimpleObject, Debug)]
pub struct Report {
    /// The sessions included in this report
    pub sessions: Vec<Session>,
}

impl Report {
    pub async fn for_range_inclusive(
        pool: &Pool<Sqlite>,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Self> {
        let sessions = Session::for_range_inclusive(&pool, start, end).await?;

        Ok(Self { sessions })
    }
}
