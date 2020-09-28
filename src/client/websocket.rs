use seed::{
    *,
    prelude::*,
    prelude::WebSocket as SeedWebSocket,
};
use components::{
    Component,
    Init,
};
use crate::{
    shared::{
        self,
        ClientMessage,
        ServerMessage,
    },
};
use tracing::{
    debug,
};
use crate::chart;

#[derive(Debug)]
pub struct WebSocket {
    pub host: String,
    pub websocket: Option<SeedWebSocket>,
    pub websocket_reconnector: Option<StreamHandle>,
}
impl Init<String> for WebSocket {
    fn init(host: String, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ServerMessage|
            Msg::SendWebSocketMessage(msg));
        Self {
            host: host.clone(),
            websocket: Some(Self::create_websocket(&host, orders)),
            websocket_reconnector: None,
        }
    }
}
impl WebSocket {
    fn create_websocket(host: &str, orders: &mut impl Orders<Msg>) -> SeedWebSocket {
        let msg_sender = orders.msg_sender();
        let url = format!("ws://{}/ws", host);
        SeedWebSocket::builder(url, orders)
            .on_open(|| Msg::WebSocketOpened)
            .on_message(move |msg| Self::receive_message(msg, msg_sender))
            .on_close(Msg::WebSocketClosed)
            .on_error(|| Msg::WebSocketError("WebSocket failed.".to_string()))
            .build_and_open()
            .unwrap()
    }
    fn receive_message(message: WebSocketMessage, msg_sender: std::rc::Rc<dyn Fn(Option<Msg>)>) {
        if message.contains_text() {
            let msg = message
                .json::<shared::ClientMessage>()
                .expect("Failed to decode WebSocket text message");
    
            msg_sender(Some(Msg::ServerMessageReceived(msg)));
        } else {
            spawn_local(async move {
                let bytes = message
                    .bytes()
                    .await
                    .expect("WebsocketError on binary data");
    
                let msg: shared::ClientMessage = serde_json::de::from_slice(&bytes).unwrap();
                msg_sender(Some(Msg::ServerMessageReceived(msg)));
            });
        }
    }
    fn send_message(&self, msg: ServerMessage, orders: &mut impl Orders<Msg>) {
        self.websocket.as_ref().map(|ws|
            ws.send_json(&msg)
                .unwrap_or_else(|err| {
                    orders.send_msg(Msg::WebSocketError(format!("{:?}", err)));
                })
        );
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketError(String),
    ServerMessageReceived(ClientMessage),
    SendWebSocketMessage(ServerMessage),
    ReconnectWebSocket,
}
impl Component for WebSocket {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        debug!("Root update");
        match msg {
            Msg::WebSocketOpened => {
                debug!("WebSocket opened");
                orders.notify(chart::Msg::SubscribePriceHistory);
            },
            Msg::WebSocketClosed(event) => {
                debug!("WebSocket closed: {:#?}", event);
                self.websocket = None;
                if !event.was_clean() && self.websocket_reconnector.is_none() {
                    self.websocket_reconnector = Some(
                        orders.stream_with_handle(streams::backoff(None, |_| Msg::ReconnectWebSocket))
                    );
                }
            },
            Msg::WebSocketError(err) => {
                debug!("WebSocket error: {:#?}", err);
            },
            Msg::ReconnectWebSocket => {
                self.websocket = Some(Self::create_websocket(&self.host, orders));
            },
            Msg::SendWebSocketMessage(msg) => {
                //debug!("Send ServerMessage");
                //debug!("{:#?}", msg);
                self.send_message(msg, orders);
            },
            Msg::ServerMessageReceived(msg) => {
                //debug!("ClientMessage received");
                //debug!("{:#?}", msg);
                orders.notify(msg);
            },
        }
    }
}
