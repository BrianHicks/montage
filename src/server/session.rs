use super::error::{Error, Result};
use super::kind::Kind;
use async_graphql::SimpleObject;
use chrono::{DateTime, Duration, Local};
use indoc::indoc;
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

#[derive(SimpleObject, Debug, PartialEq, Eq)]
pub struct Session {
    pub id: i64,
    pub kind: Kind,
    pub description: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub end_time: Option<DateTime<Local>>,
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
                .map_err(Error::QueryError)?;

        tracing::debug!(
            count = closed_existing_sessions_receipt.rows_affected(),
            "closed existing sessions"
        );

        sqlx::query_as::<_, Session>(indoc! {"
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
        .map_err(Error::QueryError)
    }

    pub async fn current_session(pool: &Pool<Sqlite>) -> Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            indoc! {"
                SELECT *
                FROM sessions
                ORDER BY start_time DESC
                LIMIT 1
            "}
        )
        .fetch_optional(pool)
        .await
        .map_err(Error::QueryError)
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
}
