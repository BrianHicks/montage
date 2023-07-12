use chrono::Duration;

/// What kind of session are we going to have?
#[derive(async_graphql::Enum, Debug, PartialEq, Eq, Copy, Clone, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Kind {
    /// A longer session focused on doing something
    Task,

    /// A shorter recovery session
    Break,

    /// A longer recovery session like lunch, overnight, or the weekend
    NotWorking,
}

impl Kind {
    pub fn default_session_length(&self) -> Duration {
        match self {
            Self::Task => Duration::minutes(25),
            Self::Break => Duration::minutes(5),
            Self::NotWorking => Duration::minutes(60),
        }
    }
}
