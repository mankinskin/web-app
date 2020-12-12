use crate::{
    *,
    binance::{
        PriceHistoryRequest,
        Binance,
    },
};
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
    Request,
    Body,
    Response,
};
use futures_util::{
    StreamExt,
    SinkExt,
};
use shared::{
    PriceSubscription,
};
use app_model::{
    user::User,
    auth::{
        Credentials,
        login,
        register,
    }
};
use chrono::Utc;

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
async fn wss(request: Request<()>) -> tide::Result {
    WebSocket::new(async move |_, ws| {
        let (sink, stream) = ws.split();
        let stream = stream.map(|msg| msg.map(|msg| msg.into_data()));
        let sink = sink.with(async move |msg| Ok(Message::from(msg)) as Result<_, tide_websockets::Error>);
        websocket::connection(stream, sink).await;
        Ok(())
    })
    .call(request)
    .await
}
fn root() -> std::io::Result<tide::Server<()>> {
    let mut root = tide::new();
    root.at("/").get(client_file!("index.html"));
    root.at("/favicon.ico").get(client_file!("favicon.ico"));
    root.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH))?;
    Ok(root)
}
async fn login_handler(mut req: Request<()>) -> tide::Result {
    let credentials: Credentials = req.body_json().await?;
    match login::<database::Schema>(credentials).await {
        Ok(session) => {
            req.session_mut()
                .insert("session", session)
                .map(|_| Response::new(200))
                .map_err(|e| tide::Error::from_str(500, e.to_string()))
        },
        Err(e) => Err(e)
    }
}
async fn logout_handler(mut req: Request<()>) -> tide::Result {
    req.session_mut().remove("session");
    Ok(Response::new(200))
}
async fn registration_handler(mut req: Request<()>) -> tide::Result {
    let user: User = req.body_json().await?;
    match register::<database::Schema>(user).await {
        Ok(_session) => Ok(Response::new(200)),
        Err(e) => Err(tide::Error::from_str(500, e.to_string()))
    }
}
async fn post_subscription_handler(mut req: Request<()>) -> tide::Result<Body> {
    let _: PriceSubscription = req.body_json().await?;
    Ok(Body::from_json(&String::from(""))?)
}
fn subscriptions_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/").post(post_subscription_handler);
    Ok(api)
}
fn auth_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/login").post(login_handler);
    api.at("/register").post(registration_handler);
    api.at("/logout").post(logout_handler);
    Ok(api)
}
fn api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/auth").nest(auth_api()?);
    api.at("/subscriptions").nest(subscriptions_api()?);
    api.at("/price_history").nest(price_api()?);
    Ok(api)
}
async fn price_history_handler(_: Request<()>) -> tide::Result<Body> {
    match Binance::get_symbol_price_history(PriceHistoryRequest {
            market_pair: "SOLBTC".into(),
            interval: Some(openlimits::model::Interval::OneHour),
            paginator: None,
        })
        .await
    {
        Ok(data) => Ok(Body::from_json(&data)?),
        Err(e) => Err(tide::Error::from_str(500, e.to_string()))
    }
}
fn price_api() -> std::io::Result<tide::Server<()>> {
    let mut api = tide::new();
    api.at("/").get(price_history_handler);
    Ok(api)
}

fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
    tide::sessions::SessionMiddleware::new(
        tide::sessions::MemoryStore::new(),
        session::generate_secret().as_bytes()
    )
    .with_cookie_name("session")
    .with_session_ttl(Some(std::time::Duration::from_secs(session::EXPIRATION_SECS as u64)))
}

//use crate::subscriptions::actor::SubscriptionsActor;
pub async fn run() -> std::io::Result<()> {

    //let binance_actor = Binance::init();
    //let mut subscription_actor = crate::actor_sys_mut().await.actor_of_args::<SubscriptionsActor>("subscriptions-actor").unwrap();
    let mut server = tide::new();
    server.with(TraceMiddleware::new());
    server.with(session_middleware());
    server.with(tide::utils::Before(async move |mut request: Request<()>| {
        let session = request.session_mut();
        if let Some(expiry) = session.expiry() {
            // time since expiry or (negative) until
            let dt = (Utc::now() - *expiry).num_seconds();
            if dt >= session::STALE_SECS as i64 {
                // expired and stale
                session.destroy()
            } else if dt >= 0 {
                // expired and not stale
                session.regenerate()
            }
        }
        request
    }));
    server.at("/").nest(root()?);
    server.at("/api").nest(api()?);
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
