use shared::{
    ClientMessage,
    ServerMessage,
};
use components::{
    Component,
    Init,
};
use seed::{
    prelude::*,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
};
use crate::websocket::{
    self,
    WebSocket,
};

#[derive(Debug)]
pub struct WebApi {
    websocket: Option<WebSocket>,
    send_sub: SubHandle,
}
#[derive(Clone, Debug)]
pub enum Msg {
    WebSocket(websocket::Msg),
    SendMessage(ClientMessage),
    MessageReceived(ServerMessage),
}
impl Init<Option<WebSocket>> for WebApi {
    fn init(websocket: Option<WebSocket>, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            websocket,
            send_sub: orders.subscribe_with_handle(Msg::SendMessage),
        }
    }
}
impl Init<WebSocket> for WebApi {
    fn init(ws: WebSocket, orders: &mut impl Orders<Msg>) -> Self {
        Self::init(Some(ws), orders)
    }
}
impl Init<()> for WebApi {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        Self::init(None, orders)
    }
}
impl WebApi {
    pub fn start_websocket(&mut self, orders: &mut impl Orders<Msg>) {
        self.websocket = Some(WebSocket::init((), &mut orders.proxy(Msg::WebSocket)));
    }
}
impl Component for WebApi {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::WebSocket(msg) => {
                debug!("Sending ClientMessage");
                //debug!("{:#?}", msg);
                if let websocket::Msg::MessageReceived(m) = &msg {
                    self.update(Msg::MessageReceived(m.clone()), orders);
                }
                if let Some(ws) = &mut self.websocket {
                    ws.update(msg, &mut orders.proxy(Msg::WebSocket));
                }
            }
            Msg::SendMessage(msg) => {
                debug!("Sending ClientMessage");
                //debug!("{:#?}", msg);
                if let Some(ws) = &mut self.websocket {
                    ws.update(websocket::Msg::SendMessage(msg), &mut orders.proxy(Msg::WebSocket));
                } else {

                }
            }
            Msg::MessageReceived(msg) => {
                //debug!("ServerMessage received");
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
