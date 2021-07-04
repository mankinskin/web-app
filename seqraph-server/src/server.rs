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
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
use const_format::formatcp;
use async_std::net::SocketAddr;

const CLIENT_PATH: &str = "../client";
const PKG_PATH: &str = formatcp!("{}/pkg", CLIENT_PATH);

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
    () => {
        client_file!("index.html")
    };
}
struct TideServer {
    server: tide::Server<()>,
}
impl TideServer {
    pub fn new() -> Self {
        let mut new = Self {
            server: tide::new(),
        };
        new.server.with(TraceMiddleware::new());
        //server.with(Self::session_middleware());
        //server.with(Self::session_validator_middleware());
        new.api();
        //new.wss();
        new.root();
        new
    }
    //fn auth(server: &mut tide::Server<()>) {
    //    <Self as ServeSession<'_, Schema>>::serve(server)
    //}
    //fn wss(&mut self) {
    //    let server = &mut self.server;
    //    server.at("/wss").get(Self::wss_handler);
    //}
    fn root(&mut self) {
        //self.server.at("/subscriptions").get(index!());
        //self.server.at("/login").get(index!());
        //self.server.at("/register").get(index!());
        self.server.at("/").get(client_file!("index.html"));
        self.server.at("/favicon.ico").get(client_file!("favicon.ico"));
        self.server.at("/package.js").get(client_file!("pkg/package.js"));
        self.server.at("/package_bg.wasm").get(client_file!("pkg/package_bg.wasm"));
        //self.server.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH)).expect("Cannot serve directory");
    }
    fn api(&mut self) {
        let server = &mut self.server;
        //let route = <Route as Router<ApiRoute>>::route_sub(ApiRoute::default()).prefix();
        //debug!("Routing {}", route);
        //let mut api = tide::new();
        //Self::auth(&mut api);
        //<Self as ServeTable<'_, Route, PriceSubscription, Schema>>
        //    ::serve(&mut api);
        //api.at("/price_history").nest(price_api());
        //server.at(&route).nest(api);
    }
    //async fn wss_handler(request: Request<()>) -> tide::Result {
    //    WebSocket::new(async move |_, ws| {
    //        websocket::connection(ws).await;
    //        Ok(())
    //    })
    //    .call(request).await
    //}
    //fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
    //    tide::sessions::SessionMiddleware::new(
    //        tide::sessions::MemoryStore::new(),
    //        session_service::generate_secret().as_bytes(),
    //    )
    //    .with_cookie_name("session")
    //    .with_session_ttl(Some(std::time::Duration::from_secs(
    //        session_service::EXPIRATION_SECS as u64,
    //    )))
    //}
    //fn session_validator_middleware() -> impl Middleware<()> {
    //    tide::utils::Before(async move |mut request: Request<()>| {
    //        let session = request.session_mut();
    //        if let Some(expiry) = session.expiry() {
    //            // time since expiry or (negative) until
    //            let dt = (Utc::now() - *expiry).num_seconds();
    //            if dt >= session_service::STALE_SECS as i64 {
    //                // expired and stale
    //                session.destroy()
    //            } else if dt >= 0 {
    //                // expired and not stale
    //                session.regenerate()
    //            }
    //        }
    //        request
    //    })
    //}
    pub async fn listen(self, addr: SocketAddr) -> std::io::Result<()> {
        self.server.listen(
            addr
            //TlsListener::build()
            //    .addrs(addr)
            //    .cert(keys::to_key_path("tls.crt"))
            //    .key(keys::to_key_path("tls.key")),
        )
        .await
    }
}

pub async fn run() -> std::io::Result<()> {
    TideServer::new()
        .listen(SocketAddr::from(([0, 0, 0, 0], 8000)))
        .await
}