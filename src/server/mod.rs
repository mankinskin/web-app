pub mod binance;
pub mod command;
pub mod error;
pub mod keys;
pub mod subscriptions;
pub mod telegram;
pub mod websocket;
pub mod database;

use crate::{
    binance::PriceHistoryRequest,
};
use app_model::{
    auth::{
        credentials::Credentials,
        self,
    },
    user::User,
};
use async_std::net::SocketAddr;
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    warn,
    trace,
};
use actix_files::{
    NamedFile,
    Files,
};
use actix_web::{
    get,
    post,
    web,
    App,
    HttpServer,
    HttpRequest,
    Responder,
};
use openssl::ssl::{
    SslFiletype,
    SslAcceptor,
    SslMethod,
};
use actix_web_actors::ws;
use std::fmt::{
    Formatter,
    Display,
    self,
};
const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{
        Layer,
        Subscriber,
    },
    layer::SubscriberExt,
};
#[derive(Debug, Clone)]
pub struct Error(String);
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}
#[get("/ws")]
async fn ws_route(request: HttpRequest, stream: web::Payload) -> impl Responder {
    info!("Websocket session request");
    ws::start(websocket::Session::new(), &request, stream)
}
async fn index() -> impl Responder {
    info!("Index request");
    NamedFile::open(format!("{}/index.html", PKG_PATH))
}
#[get("/price_history")]
async fn price_history() -> impl Responder {
    crate::binance::Binance::get_symbol_price_history(PriceHistoryRequest {
            market_pair: "SOLBTC".into(),
            interval: Some(openlimits::model::Interval::OneHour),
            paginator: None,
        })
        .await
        .map(|data| serde_json::to_string(&data).unwrap())
}
#[post("/login")]
async fn login(credentials: web::Json<Credentials>) -> impl Responder {
    auth::login::<database::Schema>(credentials.into_inner()).await
        .map(|session| web::Json(session))
}
#[post("/register")]
async fn register(user: web::Json<User>) -> impl Responder {
    auth::register::<database::Schema>(user.into_inner())
        .await
        .map(|session| web::Json(session))
}
fn init_tracing() -> WorkerGuard {
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
pub async fn run() -> std::io::Result<()> {
    let _tracing = init_tracing();
    let _telegram = telegram::Telegram::init().await;
    let _cli = command::CommandLine::init().await;
    let binance = binance::Binance::init().await;
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let mut ssl_builder = SslAcceptor::mozilla_modern(SslMethod::tls())?;
    ssl_builder.set_certificate_chain_file("./keys/tls.crt")?;
    ssl_builder.set_private_key_file("./keys/tls.key", SslFiletype::PEM)?;
    let server = HttpServer::new(move ||
            App::new()
                .wrap(tracing_actix_web::TracingLogger)
                .data(binance.clone())
                .route("/", web::get().to(index))
                .route("/subscriptions", web::get().to(index))
                .route("/login", web::get().to(index))
                .route("/register", web::get().to(index))
                .service(
                    web::scope("/api")
                        .service(price_history)
                        .service(login)
                        .service(register)
                )
                .service(ws_route)
                .service(Files::new("/", PKG_PATH))
        )
        .bind_openssl(addr, ssl_builder)?;
    info!("Starting Server");
    server.run().await
}
