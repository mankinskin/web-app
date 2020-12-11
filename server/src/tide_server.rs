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
use tide_websockets::{
    Message,
    WebSocket,
};
use tide::{
    Endpoint,
};
use futures_util::{
    StreamExt,
    SinkExt,
};

macro_rules! client_file {
    ($path:expr) => {
        |_: tide::Request<()>| async move {
            let body = tide::Body::from_file(format!("{}/{}", CLIENT_PATH, $path)).await?;
            Ok(tide::Response::from(body))
        }
    }
}
macro_rules! index {
    () => { client_file!("index.html") }
}
async fn wss(request: tide::Request<()>) -> tide::Result {
    WebSocket::new(async move |_, ws| {
        let (sink, stream) = ws.split();
        let stream = stream.map(|msg| msg.map(|msg| msg.into_data()));
        let sink = sink.with(async move |msg| Ok(Message::from(msg)) as Result<_, tide_websockets::Error>);
        websocket::websocket_session(stream, sink).await;
        Ok(())
    })
    .call(request)
    .await
}

pub async fn run() -> std::io::Result<()> {
    let mut server = tide::new();
    server.with(TraceMiddleware::new());
    server.at("/").nest({
        let mut root = tide::new();
        root.at("/").get(client_file!("index.html"));
        root.at("/favicon.ico").get(client_file!("favicon.ico"));
        root.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH))?;
        root
    });
    server.at("/subscriptions").get(index!());
    server.at("/login").get(index!());
    server.at("/register").get(index!());
    server.at("/wss").get(wss);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    server.listen(TlsListener::build()
            .addrs(addr)
            .cert(keys::to_key_path("tls.crt"))
            .key(keys::to_key_path("tls.key")),
        ).await?;
    Ok(())
}
