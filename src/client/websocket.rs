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
    websocket: Option<SeedWebSocket>,
    websocket_reconnector: Option<StreamHandle>,
    send_sub: SubHandle,
}
impl Init<String> for WebSocket {
    fn init(host: String, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            host: host.clone(),
            websocket: Some(Self::create_websocket(&host, orders)),
            websocket_reconnector: None,
            send_sub: orders.subscribe_with_handle(Msg::SendMessage),
        }
    }
}
impl WebSocket {
    fn create_websocket(host: &str, orders: &mut impl Orders<Msg>) -> SeedWebSocket {
        debug!("Creating websocket");
        let msg_sender = orders.msg_sender();
        let url = format!("wss://{}/wss", host);
        let ws = SeedWebSocket::builder(url, orders)
            .on_open(|| Msg::Opened)
            .on_message(move |msg| Self::receive_message(msg, msg_sender))
            .on_close(Msg::Closed)
            .on_error(|| Msg::Error("WebSocket failed.".to_string()))
            .build_and_open()
            .expect("Failed to build WebSocket");
        debug!("Built websocket");
        ws
    }
    fn receive_message(message: WebSocketMessage, msg_sender: std::rc::Rc<dyn Fn(Option<Msg>)>) {
        //debug!("Receiving message");
        if message.contains_text() {
            let msg = message
                .json::<shared::ClientMessage>()
                .expect("Failed to decode WebSocket text message");
    
            msg_sender(Some(Msg::MessageReceived(msg)));
        } else {
            spawn_local(async move {
                let bytes = message
                    .bytes()
                    .await
                    .expect("websocket::Error on binary data");
    
                let msg: shared::ClientMessage = serde_json::de::from_slice(&bytes).unwrap();
                msg_sender(Some(Msg::MessageReceived(msg)));
            });
        }
    }
    fn send_message(&self, msg: ServerMessage, orders: &mut impl Orders<Msg>) {
        //debug!("Sending message");
        self.websocket.as_ref().map(|ws|
            ws.send_json(&msg)
                .unwrap_or_else(|err| {
                    orders.send_msg(Msg::Error(format!("{:?}", err)));
                })
        );
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Opened,
    Closed(CloseEvent),
    Error(String),
    Reconnect,
    MessageReceived(ClientMessage),
    SendMessage(ServerMessage),
}
impl Component for WebSocket {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        //debug!("Websocket update");
        match msg {
            Msg::Opened => {
                debug!("WebSocket opened");
                orders.notify(chart::Msg::SubscribePriceHistory);
            },
            Msg::Closed(event) => {
                debug!("WebSocket closed: {:#?}", event);
                self.websocket = None;
                if !event.was_clean() && self.websocket_reconnector.is_none() {
                    self.websocket_reconnector = Some(
                        orders.stream_with_handle(streams::backoff(None, |_| Msg::Reconnect))
                    );
                }
            },
            Msg::Error(err) => {
                debug!("WebSocket error: {:#?}", err);
            },
            Msg::Reconnect => {
                debug!("Reconnect websocket");
                self.websocket = Some(Self::create_websocket(&self.host, orders));
            },
            Msg::SendMessage(msg) => {
                //debug!("Send ServerMessage");
                //debug!("{:#?}", msg);
                self.send_message(msg, orders);
            },
            Msg::MessageReceived(msg) => {
                //debug!("ClientMessage received");
                //debug!("{:#?}", msg);
                orders.notify(msg);
            },
        }
    }
}
