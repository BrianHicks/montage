use super::session::Session;

pub type Result<Whatever> = std::result::Result<Whatever, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("query error: {0}")]
    Query(sqlx::Error),

    #[error("context error: {0:?}")]
    Context(async_graphql::Error),

    #[error("there is no current session")]
    NoCurrentSession,

    #[error("error sending new session: {0}")]
    SendError(tokio::sync::watch::error::SendError<Option<Session>>),

    #[error("validation error starting a session: {0}")]
    StartSessionError(StartSessionError),
}

#[derive(Debug, thiserror::Error)]
pub enum StartSessionError {
    #[error("description cannot be blank")]
    DescriptionWasBlank,
}
