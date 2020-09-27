pub mod connection;
pub mod connections;
pub use connections::Connections;

use warp::{
    ws::{
        WebSocket,
    },
};
use crate::{
    shared::{
        ServerMessage,
        ClientMessage,
    },
};
use futures::{
    StreamExt,
    SinkExt,
    channel::mpsc::{
        channel,
    },
};
use std::{
    convert::{
        TryInto,
    },
};
#[allow(unused)]
use tracing::{
    debug,
    error,
};
use connection::Connection;

pub async fn connection(websocket: WebSocket) {
    let (ws_server_sender, ms_server_receiver) = channel(100); // ServerMessages
    let (ms_client_sender, ws_client_receiver) = channel(100); // ClientMessages
    let id = Connections::add(Connection::new(ms_client_sender, ms_server_receiver)).await;
    // get websocket sink and stream
    let (ws_sink, ws_stream) = websocket.split();
    // forward websocket stream to message sink
    let receiver_handle = tokio::spawn(async {
        ws_stream
            .map(|msg: Result<warp::ws::Message, warp::Error>| {
                msg.map_err(Into::into)
            })
            .forward(
                ws_server_sender
                    .with(|msg: warp::ws::Message| async { 
                        msg.try_into() as Result<ServerMessage, _>
                    })
            ).await.expect("Failed to forward websocket receiving stream")
    });
    if let Ok(()) = ws_client_receiver
        .filter_map(|msg: ClientMessage| async {
            msg.try_into().map(Ok).ok()
        })
        .forward(ws_sink).await {}
    receiver_handle.await.expect("Failed to join websocket receiver thread");
    Connections::remove(id).await;
}


