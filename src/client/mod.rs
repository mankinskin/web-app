use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    View,
};
use openlimits::{
    model::{
        Candle,
    },
};
use rust_decimal::prelude::ToPrimitive;
use crate::{
    shared::{
        self,
        ClientMessage,
        ServerMessage,
        PriceHistoryRequest,
    },
};
use tracing::{
    debug,
};

const WS_URL: &str = "ws://localhost:8000/ws";

fn init_tracing() {
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
}
#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_tracing();
    App::start("app",
                |_url, orders| {
                    orders.after_next_render(|_| Msg::Init); 
                    Model {
                        view_x: 0,
                        view_y: 0,
                        width: 500,
                        height: 200,
                        data: Vec::new(),
                        error: None,
                        data_min: 0.0,
                        data_max: 0.0,
                        x_interval: 2,
                        y_interval: 0,
                        y_factor: 0.0,
                        svg_node: None,
                        websocket_reconnector: None,
                        websocket: Model::create_websocket(orders),
                        data_range: 0.0,
                    }
               },
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Debug)]
pub struct Model {
    pub websocket: WebSocket,
    pub websocket_reconnector: Option<StreamHandle>,
    pub view_x: i32,
    pub view_y: i32,
    pub width: u32,
    pub height: u32,
    pub error: Option<String>,
    pub data_min: f32,
    pub data_max: f32,
    pub y_interval: u32,
    pub y_factor: f32,
    pub data_range: f32,
    pub x_interval: u32,
    pub svg_node: Option<web_sys::Node>,
    pub data: Vec<Candle>,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Init,
    Panning(i32, i32),
    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketError,
    ServerMessageReceived(ClientMessage),
    SendWebSocketMessage(ServerMessage),
}
impl Msg {

    pub fn get_price_history() -> Self {
        Msg::SendWebSocketMessage(ServerMessage::GetPriceHistory(
            PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: Some(openlimits::model::Interval::OneHour),
                paginator: None,
            }
        ))
    }
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::Init => {
                debug!("Init");
                //self.mutation_observer();
            }
            Msg::Panning(x, y) => {
                self.view_x += x;
                self.view_y += y;
            },
            Msg::WebSocketOpened => {
                debug!("WebSocket opened");
                orders.send_msg(Msg::get_price_history());
            },
            Msg::WebSocketClosed(event) => {
                debug!("WebSocket closed: {:#?}", event);
            },
            Msg::WebSocketError => {
                debug!("WebSocket failed");
            },
            Msg::ServerMessageReceived(msg) => {
                debug!("ClientMessage received");
                //debug!("{:#?}", msg);
                match msg {
                    ClientMessage::PriceHistory(candles) => {
                        self.data = candles;
                        self.update_values();
                    },
                }
            }
            Msg::SendWebSocketMessage(msg) => {
                debug!("Send ServerMessage: {:#?}", msg);
                self.websocket.send_json(&msg).unwrap();
            }
        }
    }
}
impl Model {
    fn graph_view(&self) -> Node<<Self as Component>::Msg> {
        div![
            style!{
                St::Overflow => "scroll",
                //St::OverflowY => "auto",
                //St::Height => "auto",
                //St::Cursor => "move",
                //St::Resize => "horizontal",
            },
            ev(Ev::Click, |event| {
                event.prevent_default();
                let event = to_mouse_event(&event);
                let x = event.movement_x();
                let y = event.movement_y();
                log!("Panning {} {}", x, y);
                Msg::Panning(x, y)
            }),
            svg![
                attrs!{
                    At::ViewBox => format!("{} {} {} {}", self.view_x, self.view_y, self.width, self.height),
                    At::PreserveAspectRatio => "xMinYMin meet",
                    At::Width => self.width,
                    At::Height => self.height,
                    At::Id => "graph-svg",
                    At::Overflow => "scroll",
                },
                style!{
                    St::BackgroundColor => "gray";
                    St::Width => "100%",
                },
                self.plot_view(),
                //self.vertical_lines(),
                //self.horizontal_lines(),
            ],
        ]
    }
    pub fn create_websocket(orders: &impl Orders<Msg>) -> WebSocket {
        let msg_sender = orders.msg_sender();
        WebSocket::builder(WS_URL, orders)
            .on_open(|| Msg::WebSocketOpened)
            .on_message(move |msg| Self::decode_message(msg, msg_sender))
            .on_close(Msg::WebSocketClosed)
            .on_error(|| Msg::WebSocketError)
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
    async fn fetch_candles() -> Result<Vec<Candle>, FetchError> {
        let req = seed::fetch::Request::new("http://localhost:8000/api/price_history")
            .method(Method::Get);
        seed::fetch::fetch(req)
            .await?
            .check_status()?
            .json()
            .await
    }
    fn update_values(&mut self) {
        self.data_max = self.data.iter().map(|candle| candle.high).max().map(|d| d.to_f32().unwrap()).unwrap_or(0.0);
        self.data_min = self.data.iter().map(|candle| candle.low).min().map(|d| d.to_f32().unwrap()).unwrap_or(0.0);
        self.data_range = self.data_max-self.data_min;
        if self.data_range != 0.0 {
            self.y_factor = self.height as f32/self.data_range;
        }
        self.y_interval = (0.000001*self.y_factor).round() as u32;
        //log!(self)
    }
    fn vertical_lines(&self) -> Vec<Node<<Self as Component>::Msg>> {
        (0..(self.width/10)).fold(Vec::with_capacity(self.width as usize/10*2), |mut acc, index| {
            let x = index*10;
            let y = self.height as i32;
            let h = 10;
            let padding = 10;

            let center_dir = (-1 * self.height as i32/2).clamp(-1, 1);
            acc.push(path![
                    attrs!{
                        At::D => format!("M {} {} V {}", x, y, y + center_dir * h)
                        At::Stroke => "black"
                    }
                ]);
            if index % 10 == 0 {
                acc.push(text![
                    attrs!{
                        At::X => x,
                        At::Y => y + center_dir * (h + padding),
                        At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                        At::DominantBaseline => "middle",
                        At::TextAnchor => "middle",
                        At::FontSize => "9",
                        At::Fill => "black",
                    },
                    index.to_string()
                ])
            }
            acc
        })
    }
    fn to_y_pixels(&self, d: f32) -> i32 {
        (d * self.y_factor).round() as i32
    }
    fn horizontal_lines(&self) -> Vec<Node<<Self as Component>::Msg>> {
        let count: usize = self.height as usize/self.y_interval.max(1) as usize;
        (0..count)
            .fold(Vec::with_capacity(count*2), |mut acc, index| {
            let y: i32 = self.height as i32 - index as i32*self.y_interval as i32;
            let x: i32 = 0;
            let l = self.width as i32;
            let xp: i32 = 10;
            let yp: i32 = 10;

            let center_dir = (self.width as i32/2 - x).clamp(-1, 1);
            let emphathize = index % 10 == 0;
            let opacity = if emphathize { 0.3 } else { 0.1 };
            acc.push(path![
                    attrs!{
                        At::D => format!("M {} {} h {}", x, y, center_dir * l),
                        At::Stroke => "black",
                        At::StrokeWidth => 1,
                        At::Opacity => opacity,
                    }
            ]);
            if emphathize {
                acc.push(text![
                    attrs!{
                        At::X => x + xp,
                        At::Y => y + yp,
                        At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                        At::DominantBaseline => "middle",
                        At::TextAnchor => "middle",
                        At::FontSize => "9",
                        At::Fill => "black",
                    },
                    ((self.height as i32 - y)/10).to_string()
                ]);
            }
            acc
        })
    }
    fn update_button(&self) -> Node<<Self as Component>::Msg> {
        div![
            button![
                ev(Ev::Click, |_| Msg::get_price_history()),
                "Update!"
            ],
            if let Some(e) = &self.error {
                p![
                    style!{
                        St::Color => "red",
                    },
                    e
                ]
            } else { empty![] },
        ]
    }
    fn plot_view(&self) -> Vec<Node<<Self as Component>::Msg>> {
        let candles: Vec<(_, _)> =
                self.data
                    .iter()
                    .enumerate()
                    .collect();
        candles
            .iter()
            .fold(
                Vec::with_capacity(self.data.len()*2),
                |mut acc, (i, candle)| {
                    let open = candle.open.to_f32().unwrap();
                    let close = candle.close.to_f32().unwrap();
                    let high = candle.high.to_f32().unwrap();
                    let low = candle.low.to_f32().unwrap();
                    let height = self.to_y_pixels((open - close).abs());
                    let top = open.max(close);
                    let x_px = *i as u32*self.x_interval;
                    let y_px = self.height - self.to_y_pixels(top - self.data_min) as u32;
                    let color = if open > close {
                                    "red"
                                } else if open == close {
                                    "gray"
                                } else {
                                    "green"
                                };
                    acc.push(
                        rect![
                            attrs!{
                                At::X => x_px,
                                At::Y => y_px,
                                At::Width => self.x_interval,
                                At::Height => height,
                                At::Fill => color,
                            },
                        ]
                    );
                    let x_px = x_px + self.x_interval/2;
                    let y_px = self.height - self.to_y_pixels(high - self.data_min) as u32;
                    let height = self.to_y_pixels(high - low);
                    acc.push(
                        path![
                            attrs!{
                                At::D => format!("M {} {} v {}", x_px, y_px, height),
                                At::Stroke => color,
                                At::StrokeWidth => 1,
                            },
                        ]
                    );
                    acc
                }
            )
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.update_button(),
            self.graph_view(),
        ]
    }
}
