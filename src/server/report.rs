use super::error::Result;
use crate::server::Session;
use async_graphql::{ComplexObject, SimpleObject};
use chrono::{DateTime, Duration, Local, Timelike};
use sqlx::{Pool, Sqlite};

/// A report on sessons started and ended during a given day plus some statistics.
#[derive(SimpleObject, Debug)]
#[graphql(complex)]
pub struct Report {
    /// The first date with sessions
    pub start: DateTime<Local>,

    /// The last date with sessions
    pub end: DateTime<Local>,

    /// The sessions included in this report
    pub sessions: Vec<Session>,
}

#[ComplexObject]
impl Report {
    /// Aggregate totals of the time spent in sessions
    async fn totals(&self) -> Totals {
        self.get_totals()
    }
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

    pub fn get_totals(&self) -> Totals {
        todo!()
    }
}

/// Totals for each kind of session. If sessions started on one day and ended another, and the
/// start or end date would cut part of that time off, we only count to or from midnight in the
/// local time zone.
#[derive(SimpleObject)]
pub struct Totals {
    /// The total time spent in short breaks (that is, those 15 minutes or less)
    pub short_break: Duration,

    /// The total time spent in long breaks (that is, those more than 15 minutes)
    pub long_break: Duration,

    /// The total time spent on tasks
    pub task: Duration,
}
