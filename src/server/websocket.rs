use warp::{
    ws::{
        WebSocket,
    },
};
use crate::{
    shared::{
        ServerMessage,
        ClientMessage,
        PriceHistory,
        PriceHistoryRequest,
    },
    message_stream::Message,
    Error,
};
use openlimits::model::Paginator;
use futures::{
    Stream,
    StreamExt,
    Sink,
    SinkExt,
    channel::mpsc::{
        channel,
        Sender,
        Receiver,
    },
    task::{
        Poll,
        Context,
    },
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    convert::{
        TryFrom,
        TryInto,
    },
    time::Duration,
};
use async_std::{
    sync::{
        Arc,
        RwLock,
    },
    stream::{
        interval,
    },
};
use tracing::{
    debug,
};
use std::pin::Pin;
#[derive(Debug, Clone)]
struct PriceSubscription {
    market_pair: String,
    time_interval: openlimits::model::Interval,
    last_update: Option<chrono::DateTime<chrono::Utc>>,
}
impl From<String> for PriceSubscription {
    fn from(market_pair: String) -> Self {
        Self {
            market_pair,
            time_interval: openlimits::model::Interval::OneMinute,
            last_update: None,
        }
    }
}
impl PriceSubscription {
    pub async fn latest_price_history(&self) -> Result<PriceHistory, Error> {
        let paginator = self.last_update.map(|datetime| Paginator {
            start_time: Some(datetime.timestamp_millis() as u64),
            ..Default::default()
        });
        let req = PriceHistoryRequest {
            market_pair: self.market_pair.clone(),
            interval: Some(self.time_interval),
            paginator,
        };
        let candles = crate::binance().await.get_symbol_price_history(req).await?;
        Ok(PriceHistory {
            market_pair: self.market_pair.clone(),
            time_interval: self.time_interval,
            candles,
        })
    }
}

pub struct Connection {
    sender: Sender<ClientMessage>,
    receiver: Receiver<ServerMessage>,
    subscriptions: Vec<PriceSubscription>,
}
impl Connection {
    pub fn new(sender: Sender<ClientMessage>, receiver: Receiver<ServerMessage>) -> Self {
        Self {
            sender,
            receiver,
            subscriptions: Vec::new(),
        }
    }
    pub fn new_id() -> usize {
        CONNECTION_COUNT.fetch_add(1, Ordering::Relaxed)
    }
    pub async fn push_update(&mut self) -> Result<(), Error> {
        //debug!("Pushing updates");
        for subscription in self.subscriptions.clone().iter() {
            //debug!("Updating subscription {}", &subscription.market_pair);
            self.send(ClientMessage::PriceHistory(subscription.latest_price_history().await?)).await?;
        }
        Ok(())
    }
    pub async fn receive_message(&mut self, msg: ServerMessage) -> Result<(), Error> {
        //debug!("Received websocket msg");
        //debug!("{:#?}", msg);
        let response = match msg {
            ServerMessage::SubscribePrice(market_pair) => {
                //debug!("Subscribing to market pair {}", &market_pair);
                crate::model().await.add_symbol(market_pair.clone()).await?;
                crate::server::interval::set(interval(Duration::from_secs(1)));    
                let subscription = PriceSubscription::from(market_pair);
                let response = ClientMessage::PriceHistory(subscription.latest_price_history().await?);
                self.subscriptions.push(subscription);
                Some(response)
            },
            _ => None,
        };
        if let Some(response) = response {
            self.send(response).await.map_err(Into::into)
        } else {
            Ok(())
        }
    }
}
pub type ConnectionMap = HashMap<usize, Arc<RwLock<Connection>>>;
lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<ConnectionMap>> = Arc::new(RwLock::new(ConnectionMap::new()));
    static ref CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0);
}
pub async fn connection(websocket: WebSocket) {
    let (ws_server_sender, ms_server_receiver) = channel(100); // ServerMessages
    let (ms_client_sender, ws_client_receiver) = channel(100); // ClientMessages
    let id = add_connection(Connection::new(ms_client_sender, ms_server_receiver)).await;
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
                        msg.try_into()
                    })
            ).await.expect("Failed to forward websocket receiving stream")
    });
    if let Ok(()) = ws_client_receiver
        .filter_map(|msg: ClientMessage| async {
            msg.try_into().map(Ok).ok()
        })
        .forward(ws_sink).await {}
    receiver_handle.await.expect("Failed to join websocket receiver thread");
    remove_connection(id).await;
}
pub async fn add_connection(connection: Connection) -> usize {
    let id = Connection::new_id();
    debug!("Opening Websocket connection {}", id);
    (*CONNECTIONS).write().await.insert(id, Arc::new(RwLock::new(connection)));
    id
}
pub async fn remove_connection(id: usize) {
    (*CONNECTIONS).write().await.remove(&id);
    debug!("Closed WebSocket connection {}", id);
}
impl Stream for Connection {
    type Item = Result<ServerMessage, Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.receiver), cx)
            .map(|opt|
                opt.map(Ok))
    }
}
impl Sink<ClientMessage> for Connection {
    type Error = Error;
    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn start_send(mut self: Pin<&mut Self>, item: ClientMessage) -> Result<(), Self::Error> {
        Sink::start_send(Pin::new(&mut self.sender), item).map_err(Into::into)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>> {
        Sink::poll_close(Pin::new(&mut self.sender), cx).map_err(Into::into)
    }
}
impl TryFrom<warp::ws::Message> for ServerMessage {
    type Error = Error;
    fn try_from(msg: warp::ws::Message) -> Result<Self, Self::Error> {
        if let Ok(text) = msg.to_str() {
            serde_json::de::from_str(text).map_err(Into::into)
        } else {
            if msg.is_close() {
                Ok(ServerMessage::Close)
            } else if msg.is_ping() {
                Ok(ServerMessage::Ping)
            } else if msg.is_pong() {
                Ok(ServerMessage::Pong)
            } else if msg.is_binary() {
                let bytes = msg.as_bytes().to_vec();
                Ok(ServerMessage::Binary(bytes))
            } else {
                Err(Error::WebSocket(format!("Unable to read message: {:#?}", msg)))
            }
        }
    }
}
impl TryInto<warp::ws::Message> for ClientMessage {
    type Error = Error;
    fn try_into(self) -> Result<warp::ws::Message, Self::Error> {
        Ok(warp::ws::Message::text(
            serde_json::to_string(&self)
                .map_err(Error::SerdeJson)?)
        )
    }
}
pub struct Connections;
impl Connections {
    pub async fn receive_for_connection(i: usize, msg: ServerMessage) {
        CONNECTIONS
            .write()
            .await
            .get_mut(&i)
            .expect("Connection not found")
            .write()
            .await
            .receive_message(msg)
            .await
            .expect("Error when receiving message")
    }
    pub async fn push_updates() {
        for (_, connection) in CONNECTIONS
            .write()
            .await
            .iter_mut() {
            connection.write()
                .await
                .push_update()
                .await
                .expect("Error when pushing update")
        }
    }
}
impl Stream for Connections {
    type Item = Result<crate::message_stream::Message, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Option<Self::Item>> {
        //debug!("Polling Connections");
        CONNECTIONS.try_write()
            .and_then(|mut connections| {
                //debug!("writing on {} connections", connections.len());
                connections.iter_mut()
                    .find_map(|(id, session)| { // find connection ready
                        //debug!("checking session {}", id);
                        session.try_write().and_then(|mut session| {
                            //debug!("polling session {}", id);
                            let session_poll = Stream::poll_next(Pin::new(&mut *session), cx);
                            if let Poll::Ready(opt) = session_poll {
                                //debug!("session {} ready", id);
                                opt.map(|result| {
                                    result
                                        .map(|msg| {
                                            if let ServerMessage::Close = msg {
                                            }
                                            Message::WebSocket(id.clone(), msg)
                                        })
                                        .map_err(Error::from)
                                })
                            } else {
                                None
                            }
                        })
                    })
            })
            .map(|res| Poll::Ready(Some(res)))
            .unwrap_or(Poll::Pending)
    }
}
