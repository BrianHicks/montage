use warp::Filter;

#[tokio::main]
pub async fn serve(addr: std::net::IpAddr, port: u16) {
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {name}!"));

    warp::serve(hello).run((addr, port)).await;
}
