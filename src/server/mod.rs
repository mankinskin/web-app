pub mod binance;
pub mod command;
pub mod error;
pub mod interval;
pub mod keys;
pub mod message_stream;
pub mod subscriptions;
pub mod telegram;
pub mod websocket;

use crate::shared::PriceHistoryRequest;
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
};
use warp::reply::Reply;
use warp::Filter;
const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";

#[derive(Debug, Clone)]
pub struct Error(String);
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s)
    }
}
use std::fmt::{
    Formatter,
    Display,
    self,
};
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}
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
    HttpResponse,
    HttpRequest,
    Responder,
    middleware::Logger,
};
use openssl::ssl::{
    SslFiletype,
    SslAcceptor,
    SslMethod,
};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
struct WebsocketActor;

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {

    }
}
pub async fn run() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let mut ssl_builder = SslAcceptor::mozilla_modern(SslMethod::tls())?;
    ssl_builder.set_certificate_chain_file("./keys/tls.crt")?;
    ssl_builder.set_private_key_file("./keys/tls.key", SslFiletype::PEM)?;
    let server = HttpServer::new(||
            App::new()
                .wrap(tracing_actix_web::TracingLogger)
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
#[get("/ws")]
async fn ws_route(request: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(WebsocketActor, &request, stream)
}
async fn index() -> impl Responder {
    NamedFile::open(format!("{}/index.html", PKG_PATH))
}
#[get("/price_history")]
async fn price_history() -> impl Responder {
    crate::binance()
        .await
        .get_symbol_price_history(PriceHistoryRequest {
            market_pair: "SOLBTC".into(),
            interval: Some(openlimits::model::Interval::OneHour),
            paginator: None,
        })
        .await
        .map(|data| serde_json::to_string(&data).unwrap())
}
#[post("/login")]
async fn login(credentials: web::Json<Credentials>) -> impl Responder {
    auth::login(credentials.into_inner()).await
        .map(|session| web::Json(session))
}
#[post("/register")]
async fn register(user: web::Json<User>) -> impl Responder {
    auth::register(user.into_inner()).await
        .map(|session| web::Json(session))
}
