
use async_std::{
    net::{
        SocketAddr,
    },
};
use futures::{
    FutureExt,
    StreamExt,
};
use warp::{
    Filter,
};

const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";

pub async fn websocket_connection(websocket: warp::ws::WebSocket) {
    let (tx, rx) = websocket.split();
    rx.forward(tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket error: {:?}", e);
        }
    }).await
}
pub async fn run() -> Result<(), tokio::task::JoinError> {
    let websocket = warp::path("ws")
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| {
                    ws.on_upgrade(websocket_connection)
                });
    let api = warp::path("api");
    let price_history = api.and(warp::path("price_history"))
        .and_then(|| async {
            crate::binance().await.get_symbol_price_history(crate::binance::PriceHistoryRequest {
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
    let routes = websocket
        .or(api_routes)
        .or(pkg_dir);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let server = warp::serve(routes);
    tokio::spawn(server.run(addr)).await
}
