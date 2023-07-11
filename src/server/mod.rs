mod query;

use async_graphql::http::graphiql_source;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use query::Query;
use std::convert::Infallible;
use warp::Filter;

type MontageSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[tokio::main]
pub async fn serve(addr: std::net::IpAddr, port: u16) {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
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
}
