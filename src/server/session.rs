use super::kind::Kind;
use async_graphql::SimpleObject;
use chrono::{DateTime, Duration, Local};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

#[derive(SimpleObject, Debug)]
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
    pub async fn current_session(conn: &Pool<Sqlite>) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, kind, description, start_time, duration, end_time FROM sessions LIMIT 1",
        )
        .fetch_optional(conn)
        .await
    }
}
