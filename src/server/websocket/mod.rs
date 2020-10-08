pub mod connection;
pub mod connections;
pub use connections::Connections;

use crate::{
    shared::{
        ClientMessage,
        ServerMessage,
        subscription::PriceSubscription,
    },
    Error,
};
use async_std::stream::interval;
use connection::Connection;
use futures::{
    channel::mpsc::channel,
    SinkExt,
    StreamExt,
};
use std::convert::TryInto;
use std::time::Duration;
#[allow(unused)]
use tracing::{
    debug,
    error,
};
use warp::ws::WebSocket;
use std::collections::HashMap;

pub async fn connection(websocket: WebSocket) {
    let (ws_server_sender, ms_server_receiver) = channel(100); // ClientMessages
    let (ms_client_sender, ws_client_receiver) = channel(100); // ServerMessages
    let id = Connections::add(Connection::new(ms_client_sender, ms_server_receiver)).await;
    // get websocket sink and stream
    let (ws_sink, ws_stream) = websocket.split();
    // forward websocket stream to message sink
    let receiver_handle = tokio::spawn(async {
        ws_stream
            .map(|msg: Result<warp::ws::Message, warp::Error>| msg.map_err(Into::into))
            .forward(ws_server_sender.with(|msg: warp::ws::Message| {
                async { msg.try_into() as Result<ClientMessage, _> }
            }))
            .await
            .expect("Failed to forward websocket receiving stream")
    });
    if let Ok(()) = ws_client_receiver
        .filter_map(|msg: ServerMessage| async { msg.try_into().map(Ok).ok() })
        .forward(ws_sink)
        .await
    {}
    receiver_handle
        .await
        .expect("Failed to join websocket receiver thread");
    if Connections::contains(id).await {
        Connections::remove(id).await;
    }
}

pub async fn handle_message(id: usize, msg: ClientMessage) -> Result<(), Error> {
    let response = match msg {
        ClientMessage::SubscribePrice(request) => {
            debug!("Subscribing to market pair {}", &request.market_pair);
            //crate::subscriptions().await.add_subscription(request).await?;
            //crate::server::interval::set(interval(Duration::from_secs(1)));
            //let subscription = PriceSubscription::from(request);
            //let response = ServerMessage::PriceHistory(subscription.latest_price_history().await?);
            //Some(response)
            None
        },
        ClientMessage::GetPriceSubscriptionList => {
            debug!("Getting subscription list");
            crate::server::interval::set(interval(Duration::from_secs(1)));
            let list: HashMap<usize, PriceSubscription> =
                crate::subscriptions()
                .await
                .subscriptions.clone()
                .into_iter()
                .map(|(id, cache)| (id, cache.subscription))
                .collect();
            Some(ServerMessage::SubscriptionList(list))
        },
        ClientMessage::Close => {
            Connections::remove(id).await;
            None
        }
        _ => None,
    };
    if let Some(response) = response {
        Connections::connection(id)
            .await
            .expect(&format!("Connection {} not found!", id))
            .send(response)
            .await?;
    }
    Ok(())
}
pub async fn update() -> Result<(), Error> {

    //Connections::send_all(ServerMessage::PriceHistory(history)).await;
    Ok(())
}
