use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    View,
    Init,
    auth::{
        *,
        self,
    },
};
use openlimits::{
    model::{
        Candle,
    },
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
mod chart;
use chart::{
    Chart,
};

fn init_tracing() {
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
}
fn get_host() -> Result<String, JsValue> {
    web_sys::window().unwrap().location().host()
}
#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_tracing();
    App::start("app",
        |_url, orders| {
            orders.subscribe(|msg: ServerMessage|
                Msg::SendWebSocketMessage(msg)); 
            let host = get_host().unwrap();
            debug!("Host: {}", host);
            Model {
                host: host.clone(),
                websocket_reconnector: None,
                websocket: Some(Model::create_websocket(&host, orders)),
                chart: Chart::init((), &mut orders.proxy(Msg::Chart)),
                auth: Auth::init((), &mut orders.proxy(Msg::Auth)),
            }
        },
        |msg, model, orders| model.update(msg, orders),
        View::view,
    );
}
#[derive(Debug)]
pub struct Model {
    pub host: String,
    pub websocket: Option<WebSocket>,
    pub websocket_reconnector: Option<StreamHandle>,
    pub chart: Chart,
    pub auth: Auth,
}
#[derive(Clone, Debug)]
pub enum Msg {
    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketError(String),
    ServerMessageReceived(ClientMessage),
    SendWebSocketMessage(ServerMessage),
    ReconnectWebSocket,
    Chart(chart::Msg),
    Auth(auth::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Chart(msg) => {
                self.chart.update(msg, &mut orders.proxy(Msg::Chart));
            },
            Msg::Auth(msg) => {
                self.auth.update(msg, &mut orders.proxy(Msg::Auth));
            },
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
                self.websocket.as_ref().map(|ws|
                    ws.send_json(&msg)
                        .unwrap_or_else(|err| {
                            orders.send_msg(Msg::WebSocketError(format!("{:?}", err)));
                        })
                );
            },
            Msg::ServerMessageReceived(msg) => {
                //debug!("ClientMessage received");
                //debug!("{:#?}", msg);
                orders.notify(msg);
            },
        }
    }
}
impl Model {
    pub fn create_websocket(url: &str, orders: &impl Orders<Msg>) -> WebSocket {
        let msg_sender = orders.msg_sender();
        let url = format!("ws://{}/ws", url);
        WebSocket::builder(url, orders)
            .on_open(|| Msg::WebSocketOpened)
            .on_message(move |msg| Self::decode_message(msg, msg_sender))
            .on_close(Msg::WebSocketClosed)
            .on_error(|| Msg::WebSocketError("WebSocket failed.".to_string()))
            .build_and_open()
            .unwrap()
    }
    fn decode_message(message: WebSocketMessage, msg_sender: std::rc::Rc<dyn Fn(Option<Msg>)>) {
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
    #[allow(unused)]
    fn mutation_observer(&self) {
        if let Some(node) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|document| document.get_element_by_id("graph-svg")) {
            log!("found node");
            let closure = wasm_bindgen::closure::Closure::new(|record: web_sys::MutationRecord| {
                log!("Mutation {}", record);
            });
            let function = js_sys::Function::from(closure.into_js_value());
            let observer = web_sys::MutationObserver::new(&function).unwrap();
            observer.observe(&node).unwrap();
        }
    }
    #[allow(unused)]
    async fn fetch_candles(url: &str) -> Result<Vec<Candle>, FetchError> {
        let url = format!("http://{}/api/price_history", url);
        seed::fetch::fetch(
            Request::new(url)
                .method(Method::Get)
            )
            .await?
            .check_status()?
            .json()
            .await
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        //seed::log!("App redraw!");
        div![
            self.auth.view().map_msg(Msg::Auth),
            self.chart.view().map_msg(Msg::Chart),
        ]
    }
}
