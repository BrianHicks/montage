use async_graphql::Object;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct Mutation;

#[Object]
impl Mutation {
    async fn foo(&self) -> Result<bool> {
        tracing::info!("{:#?}", self);
        Ok(true)
    }
}
