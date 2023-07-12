use super::kind::Kind;
use async_graphql::SimpleObject;
use chrono::{DateTime, Local, Duration};

#[derive(SimpleObject, Debug)]
pub struct Session {
    pub id: i64,
    pub kind: Kind,
    pub description: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub end_time: Option<DateTime<Local>>,
}
