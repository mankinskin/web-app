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
        Paginator,
        Interval,
    },
};
use rust_decimal::prelude::ToPrimitive;
use chrono::{
    Duration,
};
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
            let host = web_sys::window().unwrap().location().host().unwrap();
            debug!("Host: {}", host);
            Model {
                host: host.clone(),
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
                websocket: Some(Model::create_websocket(&host, orders)),
                data_range: 0.0,
                last_candle_update: None,
                time_interval: Interval::OneMinute,
                update_poll_interval: None,
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
    pub update_poll_interval: Option<StreamHandle>,
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
    pub last_candle_update: Option<u64>,
    pub time_interval: Interval
}
#[derive(Clone, Debug)]
pub enum Msg {
    Init,
    Panning(i32, i32),
    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketError(String),
    ServerMessageReceived(ClientMessage),
    SendWebSocketMessage(ServerMessage),
    PollUpdate,
    ReconnectWebSocket,
    GetPriceHistory,
    SetTimeInterval(Interval),
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
            Msg::SetTimeInterval(interval) => {
                if self.time_interval != interval {
                    self.time_interval = interval;
                    self.last_candle_update = None;
                    self.data.clear();
                    orders.send_msg(Msg::PollUpdate);
                }
            },
            Msg::WebSocketOpened => {
                debug!("WebSocket opened");
                orders.send_msg(Msg::PollUpdate);
            },
            Msg::GetPriceHistory => {
                debug!("GetPriceHistory");
                orders.send_msg(self.price_history_request());
            },
            Msg::PollUpdate => {
                debug!("Update Chart");
                orders.send_msg(self.price_history_request());
                self.update_poll_interval = Some(orders.stream_with_handle(
                    streams::interval(
                        self.time_interval
                            .to_duration()
                            .num_milliseconds()
                            .max(
                                Duration::minutes(1)
                                .num_milliseconds()
                            ) as u32,
                        || Msg::PollUpdate
                    )
                ));
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
            Msg::ServerMessageReceived(msg) => {
                debug!("ClientMessage received");
                //debug!("{:#?}", msg);
                match msg {
                    ClientMessage::PriceHistory(candles) => {
                        self.append_price_history(candles);
                    },
                }
            }
            Msg::SendWebSocketMessage(msg) => {
                debug!("Send ServerMessage");
                //debug!("{:#?}", msg);
                self.websocket.as_ref().map(|ws|
                    ws.send_json(&msg)
                        .unwrap_or_else(|err| {
                            orders.send_msg(Msg::WebSocketError(format!("{:?}", err)));
                        })
                );
            }
        }
    }
}
impl Model {
    fn append_price_history(&mut self, candles: Vec<Candle>) {
        if let Some(timestamp) = self.last_candle_update {
            let new_candles = candles.iter().skip_while(|candle| candle.time <= timestamp);
            debug!("Appending {} new candles.", new_candles.clone().count());
            self.data.extend(new_candles.cloned());
        } else {
            debug!("Setting {} initial candles.", candles.len());
            self.data = candles;
        }
        self.update_values();
        self.last_candle_update = self.data.last().map(|candle| candle.time);
    }
    pub fn price_history_request(&self) -> Msg {
        let paginator = self.last_candle_update.map(|timestamp| Paginator {
            start_time: Some(timestamp),
            ..Default::default()
        });
        Msg::SendWebSocketMessage(ServerMessage::GetPriceHistory(
            PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: Some(self.time_interval),
                paginator,
            }
        ))
    }
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
    fn update_button(&self) -> Node<<Self as Component>::Msg> {
        div![
            button![
                ev(Ev::Click, |_| Msg::GetPriceHistory),
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
    fn interval_selection(&self) -> Node<<Self as Component>::Msg> {
        div![
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneMinute)),
                "1m"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::ThreeMinutes)),
                "3m"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::FifteenMinutes)),
                "15m"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneHour)),
                "1h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::FourHours)),
                "4h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::SixHours)),
                "6h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::TwelveHours)),
                "12h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneDay)),
                "1d"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::ThreeDays)),
                "3d"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneWeek)),
                "1w"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneMonth)),
                "1M"
            ],
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
    fn to_y_pixels(&self, d: f32) -> i32 {
        (d * self.y_factor).round() as i32
    }
    #[allow(unused)]
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
    #[allow(unused)]
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
        div![
            self.update_button(),
            self.interval_selection(),
            self.graph_view(),
        ]
    }
}
