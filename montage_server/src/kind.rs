use chrono::Duration;
use once_cell::sync::Lazy;

/// What kind of session are we going to have?
#[derive(async_graphql::Enum, Debug, PartialEq, Eq, Copy, Clone, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Kind {
    /// A session focused on doing something
    Task,

    /// A recovery session
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

#[derive(Debug, PartialEq, Eq)]
pub enum BreakKind {
    Short,
    Long,
}

static SHORT_VS_LONG_CUTOFF: Lazy<Duration> = Lazy::new(|| Duration::minutes(15));

impl From<chrono::Duration> for BreakKind {
    fn from(value: chrono::Duration) -> Self {
        if value <= *SHORT_VS_LONG_CUTOFF {
            Self::Short
        } else {
            Self::Long
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn break_kind_short() {
        assert_eq!(BreakKind::from(Duration::zero()), BreakKind::Short)
    }

    #[test]
    fn break_kind_at_threshold() {
        assert_eq!(BreakKind::from(*SHORT_VS_LONG_CUTOFF), BreakKind::Short)
    }

    #[test]
    fn break_kind_long() {
        assert_eq!(BreakKind::from(Duration::days(1)), BreakKind::Long)
    }
}
