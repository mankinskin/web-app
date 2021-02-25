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
    trace,
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
        Self::init(Some(Self::get_websocket(orders)), orders)
    }
}
impl WebApi {
    pub fn get_websocket(orders: &mut impl Orders<Msg>) -> WebSocket {
        WebSocket::init((), &mut orders.proxy(Msg::WebSocket))
    }
    pub fn start_websocket(&mut self, orders: &mut impl Orders<Msg>) {
        debug!("Starting WebSocket...");
        self.websocket = Some(Self::get_websocket(orders));
    }
}
impl Component for WebApi {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::WebSocket(msg) => {
                //debug!("{:#?}", msg);
                if let websocket::Msg::MessageReceived(m) = &msg {
                    self.update(Msg::MessageReceived(m.clone()), orders);
                }
                if let Some(ws) = &mut self.websocket {
                    ws.update(msg, &mut orders.proxy(Msg::WebSocket));
                }
            }
            Msg::SendMessage(msg) => {
                trace!("Sending ClientMessage");
                //debug!("{:#?}", msg);
                if let Some(ws) = &mut self.websocket {
                    ws.update(websocket::Msg::SendMessage(msg), &mut orders.proxy(Msg::WebSocket));
                } else {

                }
            }
            Msg::MessageReceived(msg) => {
                trace!("ServerMessage received");
                //debug!("{:#?}", msg);
                match msg {
                    ServerMessage::Subscriptions(response) => {
                        trace!("ServerMessage::Subscription");
                        orders.notify(response)
                    },
                };
            }
        }
    }
}
