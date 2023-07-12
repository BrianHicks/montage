use super::session::Session;
use async_graphql::{Context, Object};
use color_eyre::eyre::Result;
use sqlx::{Pool, Sqlite};

pub struct Query;

#[Object]
impl Query {
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    async fn current_session(&self, context: &Context<'_>) -> sqlx::Result<Option<Session>> {
        let data = context.data::<Pool<Sqlite>>().unwrap();
        Session::current_session(data).await
    }
}
