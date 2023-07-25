#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct CurrentSessionUpdates {
    pub current_session: Option<Session>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    pub kind: Kind,
    pub start_time: DateTime,
    pub projected_end_time: DateTime,
    pub remaining_time: Option<Duration>,
}

impl Session {
    pub fn is_meeting(&self) -> bool {
        self.kind == Kind::Meeting
    }
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Task,
    Break,
    Meeting,
}

type DateTime = chrono::DateTime<chrono::Local>;
cynic::impl_scalar!(DateTime, schema::DateTime);

type Duration = iso8601::Duration;
cynic::impl_scalar!(Duration, schema::Duration);
