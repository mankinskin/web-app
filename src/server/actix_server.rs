use crate::{
    *,
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
pub async fn run() -> std::io::Result<()> {
    let _tracing = crate::server::init_tracing();
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
