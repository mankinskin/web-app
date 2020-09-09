
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
    let pkg_dir = warp::fs::dir(PKG_PATH.to_string());
    let routes = websocket.or(pkg_dir);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let server = warp::serve(routes);
    tokio::spawn(server.run(addr)).await
}
