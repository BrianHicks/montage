use super::error::{Error, Result};
use super::kind::Kind;
use async_graphql::{ComplexObject, SimpleObject};
use chrono::{DateTime, Duration, Local};
use indoc::indoc;
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

/// A session, either currently-running or historical
#[derive(SimpleObject, Debug, PartialEq, Eq, Clone)]
#[graphql(complex)]
pub struct Session {
    #[graphql(skip)]
    pub id: i64,

    /// What kind of session is this?
    pub kind: Kind,

    /// What's going on in this session?
    pub description: String,

    /// When did this session start?
    pub start_time: DateTime<Local>,

    /// How much time have we committed to this session?
    pub duration: Duration,

    /// If the session is over, when did it end?
    pub end_time: Option<DateTime<Local>>,
}

#[ComplexObject]
impl Session {
    /// When is/was the session projected to end?
    async fn projected_end_time(&self) -> DateTime<Local> {
        self.get_projected_end_time()
    }

    /// If the session is in progress, how much time is left?
    async fn remaining_time(&self) -> Option<Duration> {
        self.get_remaining_time()
    }
}

#[derive(Debug, thiserror::Error)]
enum DurationError {
    #[error("failed to parse ISO8601 duration string: {0}")]
    ParsingError(String),

    #[error("std duration was out of range: {0}")]
    OutOfRangeError(chrono::OutOfRangeError),
}

impl FromRow<'_, SqliteRow> for Session {
    fn from_row(row: &SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let duration_str = row.try_get("duration")?;
        let duration = iso8601::duration(duration_str)
            .map_err(DurationError::ParsingError)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        Ok(Self {
            id: row.try_get("id")?,
            kind: row.try_get("kind")?,
            description: row.try_get("description")?,
            start_time: row.try_get("start_time")?,
            duration: Duration::from_std(std::time::Duration::from(duration))
                .map_err(DurationError::OutOfRangeError)
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
            end_time: row.try_get("end_time")?,
        })
    }
}

impl Session {
    fn get_projected_end_time(&self) -> DateTime<Local> {
        self.start_time + self.duration
    }

    // TODO: tests for this!
    fn get_remaining_time(&self) -> Option<Duration> {
        let now = Local::now();
        let projected_end_time = self.get_projected_end_time();

        if self.end_time.is_some() {
            None
        } else if projected_end_time < now {
            Some(Duration::seconds(0))
        } else {
            Some(projected_end_time - now)
        }
    }
    
    pub async fn start(
        pool: &Pool<Sqlite>,
        kind: Kind,
        description: &str,
        start_time: DateTime<Local>,
        duration: Duration,
    ) -> Result<Self> {
        let closed_existing_sessions_receipt =
            sqlx::query("UPDATE sessions SET end_time = ? WHERE end_time IS NULL")
                .bind(start_time)
                .execute(pool)
                .await
                .map_err(Error::Query)?;

        tracing::info!(
            count = closed_existing_sessions_receipt.rows_affected(),
            "closed existing sessions"
        );

        let res = sqlx::query_as::<_, Session>(indoc! {"
            INSERT INTO sessions (kind, description, start_time, duration)
            VALUES (?, ?, ?, ?)
            RETURNING id, kind, description, start_time, duration, end_time;
        "})
        .bind(kind)
        .bind(description)
        .bind(start_time)
        .bind(duration.to_string())
        .fetch_one(pool)
        .await
        .map_err(Error::Query)?;

        tracing::info!(
            description = res.description,
            kind = ?res.kind,
            "started new session"
        );

        Ok(res)
    }

    pub async fn extend_by(pool: &Pool<Sqlite>, duration: Duration) -> Result<Self> {
        Self::update_duration(pool, |current| current.duration + duration).await
    }

    pub async fn extend_to(pool: &Pool<Sqlite>, target: DateTime<Local>) -> Result<Self> {
        Self::update_duration(pool, |current| target - current.start_time).await
    }

    async fn update_duration<F>(pool: &Pool<Sqlite>, get_new_duration: F) -> Result<Self>
    where
        F: FnOnce(&Session) -> Duration,
    {
        let mut current = match Self::current_session(pool).await? {
            Some(session) => session,
            None => return Err(Error::NoCurrentSession),
        };

        let new_duration = get_new_duration(&current);

        let receipt = sqlx::query("UPDATE sessions SET duration = ? WHERE id = ?")
            .bind(new_duration.to_string())
            .bind(current.id)
            .execute(pool)
            .await
            .map_err(Error::Query)?;

        debug_assert!(receipt.rows_affected() == 1);

        current.duration = new_duration;

        Ok(current)
    }

    pub async fn current_session(pool: &Pool<Sqlite>) -> Result<Option<Self>> {
        sqlx::query_as::<_, Self>(indoc! {"
            SELECT *
            FROM sessions
            ORDER BY start_time DESC
            LIMIT 1
        "})
        .fetch_optional(pool)
        .await
        .map_err(Error::Query)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn get_pool() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new().connect(":memory:").await.unwrap();

        sqlx::migrate!("db/migrations").run(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn current_session_gets_nothing_in_empty_database() {
        let pool = get_pool().await;

        let current = Session::current_session(&pool).await.unwrap();

        assert_eq!(current, None);
    }

    #[tokio::test]
    async fn current_session_gets_a_started_session() {
        let pool = get_pool().await;
        let now = Local::now();

        let new_session =
            Session::start(&pool, Kind::Task, "foo".into(), now, Duration::minutes(25))
                .await
                .unwrap();
        let current_session = Session::current_session(&pool).await.unwrap();

        assert_eq!(current_session, Some(new_session));
    }

    #[tokio::test]
    async fn current_session_gets_the_most_recent_session() {
        let pool = get_pool().await;
        let now = Local::now();
        let next = now + Duration::minutes(5);

        Session::start(&pool, Kind::Task, "foo".into(), now, Duration::minutes(25))
            .await
            .unwrap();

        let session_2 =
            Session::start(&pool, Kind::Task, "foo".into(), next, Duration::minutes(25))
                .await
                .unwrap();

        let current_session = Session::current_session(&pool).await.unwrap();

        assert_eq!(current_session.map(|s| s.id), Some(session_2.id))
    }

    #[tokio::test]
    async fn starting_a_new_session_closes_existing_sessions() {
        let pool = get_pool().await;
        let now = Local::now();
        let duration = Duration::minutes(25);
        let next = now + duration;

        let first_session = Session::start(&pool, Kind::Task, "foo".into(), now, duration)
            .await
            .unwrap();

        Session::start(&pool, Kind::Task, "foo".into(), next, duration)
            .await
            .unwrap();

        let first_session_refetched =
            sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
                .bind(first_session.id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_ne!(first_session_refetched.end_time, first_session.end_time);
        assert_eq!(first_session_refetched.end_time, Some(next));
    }

    #[tokio::test]
    async fn you_cant_extend_a_session_that_doesnt_exist() {
        let pool = get_pool().await;

        match Session::extend_by(&pool, Duration::minutes(5)).await {
            Err(Error::NoCurrentSession) => (),
            other => panic!("expected NoCurrentSession, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn extending_a_session_changes_the_duration() {
        let pool = get_pool().await;
        let now = Local::now();
        let duration = Duration::minutes(5);
        let extension = Duration::minutes(5);

        let original_session = Session::start(&pool, Kind::Task, "foo".into(), now, duration)
            .await
            .unwrap();

        let extended_session = Session::extend_by(&pool, extension).await.unwrap();

        assert_eq!(
            extended_session.duration,
            original_session.duration + extension
        )
    }

    #[tokio::test]
    async fn extending_a_session_to_a_date_sets_the_duration() {
        let pool = get_pool().await;
        let now = Local::now();
        let duration = Duration::minutes(5);
        let extension = Duration::minutes(5);

        let original_session = Session::start(&pool, Kind::Task, "foo".into(), now, duration)
            .await
            .unwrap();

        let extended_session = Session::extend_to(&pool, now + duration + extension)
            .await
            .unwrap();

        assert_eq!(
            extended_session.duration,
            original_session.duration + extension
        )
    }
}
