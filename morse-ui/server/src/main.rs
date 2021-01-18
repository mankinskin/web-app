#![feature(async_closure)]
use std::fmt::Debug;
use tide::{
    Body,
    Endpoint,
    Middleware,
    Request,
    Response,
};
use tide_rustls::TlsListener;
use tide_tracing::TraceMiddleware;
use tide_websockets::{
    WebSocket,
};
use async_std::net::SocketAddr;
use std::{
    convert::AsRef,
    path::Path,
};
use chrono::{
    Utc,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
	fmt::{
		Layer,
		Subscriber,
	},
	layer::SubscriberExt,
};
#[allow(unused)]
use tracing::{
	debug,
	info,
	error,
	warn,
	trace,
};
pub fn init_tracing() -> WorkerGuard {
	tracing_log::LogTracer::init().unwrap();
	let file_appender = tracing_appender::rolling::hourly("./logs", "log");
	let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
	let subscriber = Subscriber::builder()
			.with_env_filter("hyper=error,reqwest=error,h2=error,[]=debug")
			.finish()
			.with(Layer::default().with_writer(file_writer));
	tracing::subscriber::set_global_default(subscriber)
		.expect("Unable to set global tracing subscriber");
	info!("Tracing initialized.");
	info!["Info logs enabled"];
	trace!["Trace logs enabled"];
	debug!["Debug logs enabled"];
	warn!["Warning logs enabled"];
	error!["Error logs enabled"];
	guard
}
pub const KEY_PATH: &str = "../../keys";

pub fn to_key_path<P: AsRef<Path>>(path: P) -> impl AsRef<Path> {
    Path::new(KEY_PATH).join(path)
}
pub fn read_key_file<P: AsRef<Path>>(path: P) -> String {
    let path = to_key_path(path);
    std::fs::read_to_string(path.as_ref())
        .map(|s| s.trim_end_matches("\n").to_string())
        .expect(&format!("Failed to read {}", path.as_ref().display()))
}
fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
    tide::sessions::SessionMiddleware::new(
        tide::sessions::MemoryStore::new(),
        session_service::generate_secret().as_bytes(),
    )
    .with_cookie_name("session")
    .with_session_ttl(Some(std::time::Duration::from_secs(
        session_service::EXPIRATION_SECS as u64,
    )))
}
fn session_validator_middleware() -> impl Middleware<()> {
    tide::utils::Before(async move |mut request: Request<()>| {
        let session = request.session_mut();
        if let Some(expiry) = session.expiry() {
            // time since expiry or (negative) until
            let dt = (Utc::now() - *expiry).num_seconds();
            if dt >= session_service::STALE_SECS as i64 {
                // expired and stale
                session.destroy()
            } else if dt >= 0 {
                // expired and not stale
                session.regenerate()
            }
        }
        request
    })
}
async fn websocket_connection(ws: tide_websockets::WebSocketConnection) {
}
async fn wss_handler(request: Request<()>) -> tide::Result {
    WebSocket::new(async move |_, ws| {
        websocket_connection(ws).await;
        Ok(())
    })
    .call(request).await
}

pub const CLIENT_PATH: &str = "../client";
macro_rules! client_file {
    ($path:expr) => {
        |_: tide::Request<()>| {
            async move {
                let body = tide::Body::from_file(format!("{}/{}", CLIENT_PATH, $path)).await?;
                Ok(tide::Response::from(body))
            }
        }
    };
}
macro_rules! index {
    () => { client_file!("index.html") };
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _tracing_handle = init_tracing();
    let mut server = tide::new();
    server.with(session_middleware());
    server.with(session_validator_middleware());
    server.at("/").get(index!());
    server.at("/favicon.ico").get(client_file!("favicon.ico"));
    server.at("/package.js").get(client_file!("pkg/package.js"));
    server.at("/package_bg.wasm").get(client_file!("pkg/package_bg.wasm"));
    server.listen(
    TlsListener::build()
        .addrs(SocketAddr::from(([0, 0, 0, 0], 8000)))
        .cert(to_key_path("tls.crt"))
        .key(to_key_path("tls.key")),
    )
    .await
}
