use shared::{
    ClientMessage,
    ServerMessage,
};
use components::{
    Component,
    Init,
};
use seed::{
    prelude::WebSocket as SeedWebSocket,
    prelude::*,
    *,
};
use tracing::{
    debug,
    error,
};

#[derive(Debug)]
pub struct WebSocket {
    pub host: String,
    websocket: Option<SeedWebSocket>,
    websocket_reconnector: Option<StreamHandle>,
    send_sub: SubHandle,
    open: bool,
    msg_queue: Vec<ClientMessage>,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Opened,
    Closed(CloseEvent),
    Error(String),
    Reconnect,
    MessageReceived(ServerMessage),
    SendMessage(ClientMessage),
}
impl Init<()> for WebSocket {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        let host = crate::get_host().unwrap();
        //debug!("Host: {}", host);
        Self {
            host: host.clone(),
            websocket: Some(Self::create_websocket(&host, orders)),
            websocket_reconnector: None,
            send_sub: orders.subscribe_with_handle(Msg::SendMessage),
            open: false,
            msg_queue: Vec::new(),
        }
    }
}
impl WebSocket {
    fn create_websocket(host: &str, orders: &mut impl Orders<Msg>) -> SeedWebSocket {
        debug!("Creating websocket");
        let msg_sender = orders.msg_sender();
        let url = format!("wss://{}/ws", host);
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
                .json::<ServerMessage>()
                .expect("Failed to decode WebSocket text message");

            msg_sender(Some(Msg::MessageReceived(msg)));
        } else {
            spawn_local(async move {
                let bytes = message
                    .bytes()
                    .await
                    .expect("websocket::Error on binary data");

                let msg: ServerMessage = serde_json::de::from_slice(&bytes).unwrap();
                msg_sender(Some(Msg::MessageReceived(msg)));
            });
        }
    }
    fn send_message(&self, msg: ClientMessage) {
        debug!("Sending message");
        if let Some(ws) = &self.websocket {
            if let Err(err) = ws.send_json(&msg) {
                error!("{:?}", err);
            }
        }
    }
    fn push_message(&mut self, msg: ClientMessage) {
        if self.open {
            self.send_message_queue();
            self.send_message(msg);
        } else {
            self.msg_queue.push(msg);
        }
    }
    fn send_message_queue(&mut self) {
        for msg in &self.msg_queue {
            self.send_message(msg.clone());
        }
        self.msg_queue.clear();
    }
    fn start_reconnecting(&mut self, orders: &mut impl Orders<Msg>) {
        self.websocket_reconnector =
            Some(orders.stream_with_handle(streams::backoff(None, |_| Msg::Reconnect)));
    }
    fn reconnect(&mut self, orders: &mut impl Orders<Msg>) {
        self.websocket = Some(Self::create_websocket(&self.host, orders));
        self.open = true;
    }
    fn closed(&mut self, event: CloseEvent, orders: &mut impl Orders<Msg>) {
        self.websocket = None;
        if !event.was_clean() && self.websocket_reconnector.is_none() {
            self.start_reconnecting(orders);
        }
        self.open = false;
    }
    fn opened(&mut self, orders: &mut impl Orders<Msg>) {
        // notify listeners
        orders.notify(Msg::Opened);

        self.open = true;
        self.send_message_queue();
    }
}
impl Component for WebSocket {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        //debug!("Websocket update");
        match msg {
            Msg::Opened => {
                debug!("WebSocket opened");
                self.opened(orders);
            }
            Msg::Closed(event) => {
                debug!("WebSocket closed: {:#?}", event);
                self.closed(event, orders);
            }
            Msg::Error(err) => {
                debug!("WebSocket error: {:#?}", err);
            }
            Msg::Reconnect => {
                debug!("Reconnect websocket");
                self.reconnect(orders);
            }
            Msg::SendMessage(msg) => {
                debug!("Sending ClientMessage");
                //debug!("{:#?}", msg);
                self.push_message(msg);
            }
            Msg::MessageReceived(msg) => {
                debug!("ServerMessage received");
                //debug!("{:#?}", msg);
                match msg {
                    ServerMessage::Subscriptions(response) => {
                        debug!("Subscription response");
                        orders.notify(response)
                    },
                };
            }
        }
    }
}
