mod error;
mod kind;
mod mutation;
mod query;
mod report;
mod session;
mod subscription;

use async_graphql::http::graphiql_source;
use async_graphql::Schema;
use color_eyre::eyre::{Result, WrapErr};
use mutation::Mutation;
use query::Query;
use session::Session;
use sqlx::{Pool, Sqlite};
use std::convert::Infallible;
use subscription::Subscription;
use warp::Filter;

type MontageSchema = Schema<Query, Mutation, Subscription>;

pub async fn schema(pool: Pool<Sqlite>) -> Result<MontageSchema> {
    let initial = Session::current_session(&pool).await?;

    let (sender, receiver) = tokio::sync::watch::channel(initial);

    Ok(
        Schema::build(Query, Mutation::new(sender), Subscription::new(receiver))
            .extension(async_graphql::extensions::Tracing)
            .data(pool)
            .finish(),
    )
}

pub async fn serve(pool: Pool<Sqlite>, addr: std::net::IpAddr, port: u16) -> Result<()> {
    sqlx::migrate!("db/migrations")
        .run(&pool)
        .await
        .wrap_err("could not run migrations")?;

    let schema = schema(pool).await?;

    let graphql = async_graphql_warp::graphql(schema.clone()).and_then(
        |(schema, request): (MontageSchema, async_graphql::Request)| async move {
            let resp = schema.execute(request).await;
            Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
        },
    );

    let subscriptions = async_graphql_warp::graphql_subscription(schema);

    let subscriptions_url = format!("ws://{}:{}/", &addr, &port);

    let graphiql = warp::path("graphiql")
        .map(move || warp::reply::html(graphiql_source("graphql", Some(&subscriptions_url))));

    warp::serve(graphql.or(graphiql).or(subscriptions))
        .run((addr, port))
        .await;

    Ok(())
}
