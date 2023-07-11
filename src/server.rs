use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use std::convert::Infallible;
use warp::Filter;

struct Query;

#[Object]
impl Query {
    async fn version(&self) -> &'static str {
        "0.1"
    }
}

type MontageSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[tokio::main]
pub async fn serve(addr: std::net::IpAddr, port: u16) {
    tracing::info!("Listening on {addr}:{port}");

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
        .finish();

    let graphql = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (MontageSchema, async_graphql::Request)| async move {
            let resp = schema.execute(request).await;
            Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
        },
    );

    warp::serve(graphql).run((addr, port)).await;
}
