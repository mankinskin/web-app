pub mod error;
pub mod telegram;
pub mod keys;
pub mod binance;
pub mod model;
pub mod message_stream;
pub mod command;
pub mod websocket;
pub mod interval;
pub mod subscription;

use async_std::{
    net::{
        SocketAddr,
    },
};
use warp::{
    Filter,
};
#[allow(unused)]
use tracing::{
    debug,
    info,
};
use app_model::{
    user::{
        User,
    },
    auth::{
        login,
        register,
        credentials::Credentials,
    },
};
use crate::shared::{
    PriceHistoryRequest,
};
use warp::reply::Reply;
const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";

pub async fn listen() {
    let websocket = warp::path("wss")
                .and(warp::ws())
                .map(move |ws: warp::ws::Ws| {
                    debug!("Websocket connection request");
                    ws.on_upgrade(websocket::connection)
                });
    let price_history = warp::get()
        .and(warp::path!("api"/"price_history"))
        .and_then(|| async {
            crate::binance().await
                .get_symbol_price_history(PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: Some(openlimits::model::Interval::OneHour),
                paginator: None,
            })
            .await
            .map(|data| serde_json::to_string(&data).unwrap())
            .map_err(|_err|
                warp::reject::not_found()
            )
        });
    let login = warp::post()
        .and(warp::path!("api"/"login"))
        .and(warp::body::json())
        .and_then(|credentials: Credentials| async {
            Ok(match login(credentials).await {
                Ok(session) => warp::reply::json(&session).into_response(),
                Err(status) => warp::reply::with_status("", status).into_response(),
            }) as Result<warp::reply::Response, core::convert::Infallible>
        });
    let register = warp::post()
        .and(warp::path!("api"/"register"))
        .and(warp::body::json())
        .and_then(|user: User| async {
            Ok(match register(user).await {
                Ok(session) => warp::reply::json(&session).into_response(),
                Err(status) => warp::reply::with_status("", status).into_response(),
            }) as Result<warp::reply::Response, core::convert::Infallible>
        });
    let api = price_history
            .or(login)
            .or(register);
    let pkg_dir = warp::fs::dir(PKG_PATH.to_string());
    let logger = warp::log::custom(|info|
        debug!("request from {:?}: {} {} {}ms {}",
            info.remote_addr(),
            info.method(),
            info.path(),
            info.elapsed().as_millis(),
            info.status(),
        )
    );
    let routes = websocket
        .or(api)
        .or(pkg_dir)
        .or(warp::fs::file(format!("{}/index.html", PKG_PATH)))
        .with(logger);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let server = warp::serve(routes)
        .tls()
        .cert_path("./keys/tls.crt")
        .key_path("./keys/tls.key");
    info!("Starting Server");
    server.run(addr).await;
}
