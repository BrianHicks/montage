use async_tungstenite::tungstenite::{
    client::IntoClientRequest, handshake::client::Request, http::HeaderValue,
};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use cynic::http::ReqwestExt;
use cynic::{GraphQlResponse, Operation};
use serde::{de::DeserializeOwned, Serialize};

pub static DEFAULT_ADDR: &str = "127.0.0.1";

/// Squatting on a IANA reserved port of a project that I used to work on which got a reserved port
/// but (sadly) never saw real production use. It's super unlikely that I'll ever have a conflict
/// here from a system service since it's reserved!
pub static DEFAULT_PORT: &str = "4774";

#[derive(Parser, Debug)]
pub struct GraphQLClient {
    /// The address to bind to
    #[arg(long, default_value = DEFAULT_ADDR, env = "MONTAGE_ADDR")]
    server_addr: std::net::IpAddr,

    /// The port to bind to
    #[arg(long, default_value = DEFAULT_PORT, env = "MONTAGE_PORT")]
    server_port: u16,
}

impl GraphQLClient {
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}/graphql", self.server_addr, self.server_port)
    }

    pub async fn make_graphql_request<ResponseData, Vars>(
        &self,
        query: Operation<ResponseData, Vars>,
    ) -> Result<GraphQlResponse<ResponseData>>
    where
        Vars: Serialize,
        ResponseData: DeserializeOwned + 'static,
    {
        let client = reqwest::Client::new();

        client
            .post(self.endpoint())
            .run_graphql(query)
            .await
            .wrap_err("GraphQL request failed")
    }

    pub fn request(&self) -> Result<Request> {
        let mut request = format!("ws://{}:{}", self.server_addr, self.server_port)
            .into_client_request()
            .wrap_err("could not make a request with addresses provided")?;

        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );

        Ok(request)
    }
}
