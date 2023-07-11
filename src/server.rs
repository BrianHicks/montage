use juniper::{EmptyMutation, EmptySubscription, RootNode};
use warp::Filter;

struct Context {}

impl juniper::Context for Context {}

struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    fn api_version() -> &'static str {
        "0.1"
    }
}

type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    RootNode::new(
        Query,
        EmptyMutation::new(),
        EmptySubscription::new(),
    )
}

#[tokio::main]
pub async fn serve(addr: std::net::IpAddr, port: u16) {
    tracing::info!("Listening on {addr}:{port}");

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {name}!"));

    let state = warp::any().map(|| Context {});
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    let graphiql = warp::path("graphiql").and(juniper_warp::graphiql_filter("/graphql", None));
    let graphql = warp::path("graphql").and(graphql_filter);

    warp::serve(hello.or(graphiql).or(graphql))
        .run((addr, port))
        .await;
}
