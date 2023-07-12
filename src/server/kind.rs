use chrono::Duration;

/// What kind of session are we going to have?
#[derive(async_graphql::Enum, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    /// A longer session focused on doing something
    Task,

    /// A shorter recovery session
    Break,
}

impl Kind {
    pub fn default_session_length(&self) -> Duration {
        match self {
            Self::Task => Duration::minutes(25),
            Self::Break => Duration::minutes(5),
        }
    }
}
