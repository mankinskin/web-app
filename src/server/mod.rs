pub mod error;
pub mod telegram;
pub mod keys;
pub mod binance;
pub mod model;
pub mod message_stream;
pub mod command;
pub mod websocket;

use async_std::{
    net::{
        SocketAddr,
    },
};
use warp::{
    Filter,
};
use tracing::{
    debug,
    error,
};
use crate::{
    shared,
};
const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";
pub async fn listen() {
    let websocket = warp::path("ws")
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| {
                    ws.on_upgrade(move |websocket| async {
                        if let Err(e) = websocket::open_connection(websocket).await {
                            error!("{:#?}", e);
                        }
                    })
                });
    let api = warp::path("api");
    let price_history = api.and(warp::path("price_history"))
        .and_then(|| async {
            crate::binance().await
                .get_symbol_price_history(shared::PriceHistoryRequest {
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
    let api_routes = price_history;
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
        .or(api_routes)
        .or(pkg_dir)
        .with(logger);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let server = warp::serve(routes);
    debug!("Starting Server");
    server.run(addr).await
}
