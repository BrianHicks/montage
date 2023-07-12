mod mutation;
mod query;
mod session;

use async_graphql::http::graphiql_source;
use async_graphql::{EmptySubscription, Schema};
use color_eyre::eyre::Result;
use mutation::Mutation;
use query::Query;
use sqlx::sqlite;
use std::convert::Infallible;
use warp::Filter;

type MontageSchema = Schema<Query, Mutation, EmptySubscription>;

#[tokio::main]
pub async fn serve(addr: std::net::IpAddr, port: u16) -> Result<()> {
    let pool = sqlite::SqlitePoolOptions::new().connect(":memory:").await?;

    sqlx::migrate!("db/migrations").run(&pool).await?;

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
        .data(pool)
        .finish();

    let graphql = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (MontageSchema, async_graphql::Request)| async move {
            let resp = schema.execute(request).await;
            Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
        },
    );

    let graphiql =
        warp::path("graphiql").map(|| warp::reply::html(graphiql_source("graphql", None)));

    warp::serve(graphql.or(graphiql)).run((addr, port)).await;

    Ok(())
}
