use super::kind::Kind;
use super::session::Session;
use super::{error::Result, kind::BreakKind};
use async_graphql::{ComplexObject, SimpleObject};
use chrono::{DateTime, Datelike, Duration, Local, Timelike};
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
        let start_date = at_midnight(start);
        let end_date = at_midnight(end);

        let sessions = Session::for_range_inclusive(&pool, start_date, end_date).await?;

        Ok(Self {
            sessions,
            start: start_date,
            end: end_date,
        })
    }

    pub fn get_totals(&self) -> Totals {
        Totals::from_sessions(&self.sessions, self.start, self.end)
    }
}

fn at_midnight<TZ: chrono::TimeZone>(date: DateTime<TZ>) -> DateTime<TZ> {
    if date.num_seconds_from_midnight() == 0 {
        date
    } else {
        date.timezone()
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap()
    }
}

/// Totals for each kind of session. If sessions started on one day and ended another, and the
/// start or end date would cut part of that time off, we only count to or from midnight in the
/// local time zone. Incomplete sessions are included in these totals!
#[derive(SimpleObject, Debug, PartialEq, Eq)]
pub struct Totals {
    /// The total time spent in short breaks (that is, those 15 minutes or less)
    pub short_break: Duration,

    /// The total time spent in long breaks (that is, those more than 15 minutes)
    pub long_break: Duration,

    /// The total time spent on tasks
    pub task: Duration,
}

impl Default for Totals {
    fn default() -> Self {
        Totals {
            short_break: Duration::zero(),
            long_break: Duration::zero(),
            task: Duration::zero(),
        }
    }
}

impl Totals {
    fn from_sessions(
        sessions: &Vec<Session>,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Self {
        let mut totals = Self::default();

        let start_date = at_midnight(start);
        let end_date = at_midnight(end);

        for session in sessions.iter() {
            let session_total_within_dates = session.total_time_within_dates(start_date, end_date);
            debug_assert!(session_total_within_dates >= Duration::zero());

            match session.kind {
                Kind::Task => totals.task = totals.task + session_total_within_dates,
                Kind::Break => match BreakKind::from(session.get_actual_duration()) {
                    BreakKind::Short => {
                        totals.short_break = totals.short_break + session_total_within_dates
                    }
                    BreakKind::Long => {
                        totals.long_break = totals.long_break + session_total_within_dates
                    }
                },
            };
        }

        debug_assert!(totals.short_break > Duration::zero());
        debug_assert!(totals.long_break > Duration::zero());
        debug_assert!(totals.task > Duration::zero());

        totals
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn session(kind: Kind, start: DateTime<Local>, duration: Duration, ended: bool) -> Session {
        Session {
            id: 0,
            description: String::from(""),
            kind,
            start_time: start,
            duration,
            end_time: if ended { Some(start + duration) } else { None },
        }
    }

    #[test]
    fn adds_tasks() {
        let now = Local::now();

        let totals = Totals::from_sessions(
            &vec![
                session(Kind::Task, now, Duration::minutes(5), true),
                session(Kind::Task, now, Duration::minutes(5), true),
            ],
            now - Duration::days(1),
            now + Duration::days(1),
        );

        assert_eq!(
            totals,
            Totals {
                short_break: Duration::zero(),
                long_break: Duration::zero(),
                task: Duration::minutes(10),
            }
        )
    }

    #[test]
    fn adds_short_breaks() {
        let now = Local::now();

        let totals = Totals::from_sessions(
            &vec![
                session(Kind::Break, now, Duration::minutes(5), true),
                session(Kind::Break, now, Duration::minutes(5), true),
            ],
            now - Duration::days(1),
            now + Duration::days(1),
        );

        assert_eq!(
            totals,
            Totals {
                short_break: Duration::minutes(10),
                long_break: Duration::zero(),
                task: Duration::zero(),
            }
        )
    }

    #[test]
    fn adds_long_breaks() {
        let now = Local::now();

        let totals = Totals::from_sessions(
            &vec![
                session(Kind::Break, now, Duration::hours(1), true),
                session(Kind::Break, now, Duration::hours(1), true),
            ],
            now - Duration::days(1),
            now + Duration::days(1),
        );

        assert_eq!(
            totals,
            Totals {
                short_break: Duration::zero(),
                long_break: Duration::hours(2),
                task: Duration::zero(),
            }
        )
    }

    #[test]
    fn cuts_off_overnight_breaks() {
        let today = at_midnight(Local::now());

        let totals = Totals::from_sessions(
            &vec![session(
                Kind::Break,
                today - Duration::hours(8),
                Duration::hours(16),
                true,
            )],
            today,
            today + Duration::days(1),
        );

        assert_eq!(
            totals,
            Totals {
                short_break: Duration::zero(),
                long_break: Duration::hours(8),
                task: Duration::zero(),
            }
        )
    }
}
