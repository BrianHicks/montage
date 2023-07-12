pub type Result<Whatever> = std::result::Result<Whatever, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("query error: {0}")]
    QueryError(sqlx::Error),

    #[error("context error: {0:?}")]
    ContextError(async_graphql::Error),

    #[error("there is no current session")]
    NoCurrentSession,
}
