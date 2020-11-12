use crate::*;
use tide_tracing::{
    TraceMiddleware,
};
use async_std::net::SocketAddr;
use tide_rustls::TlsListener;
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    warn,
    trace,
};

async fn index(_: tide::Request<()>) -> tide::Result {
    info!("Index request");
    let body = tide::Body::from_file(format!("{}/index.html", PKG_PATH)).await?;
    Ok(tide::Response::from(body))
}
pub async fn run() -> std::io::Result<()> {
    let mut server = tide::new();
    server.with(TraceMiddleware::new());
    server.at("/").get(index);
    server.at("/subscriptions").get(index);
    server.at("/login").get(index);
    server.at("/register").get(index);
    server.at("/").serve_dir(PKG_PATH)?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    server.listen(TlsListener::build()
            .addrs(addr)
            .cert(keys::to_key_path("tls.crt"))
            .key(keys::to_key_path("tls.key")),
        ).await?;
    Ok(())
}
