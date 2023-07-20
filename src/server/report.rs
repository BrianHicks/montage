use super::error::Result;
use crate::server::Session;
use async_graphql::SimpleObject;
use chrono::{DateTime, Duration, Local, Timelike};
use sqlx::{Pool, Sqlite};

/// A report on sessons started and ended during a given day plus some statistics.
#[derive(SimpleObject, Debug)]
pub struct Report {
    /// The first date with sessions
    pub start: DateTime<Local>,

    /// The last date with sessions
    pub end: DateTime<Local>,

    /// The sessions included in this report
    pub sessions: Vec<Session>,
}

impl Report {
    pub async fn for_range_inclusive(
        pool: &Pool<Sqlite>,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Self> {
        let start_date = Self::at_midnight(start);
        let end_date = Self::at_midnight(end);

        let sessions = Session::for_range_inclusive(&pool, start_date, end_date).await?;

        Ok(Self {
            sessions,
            start: start_date,
            end: end_date,
        })
    }

    fn at_midnight(date: DateTime<Local>) -> DateTime<Local> {
        date.with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }
}

}
