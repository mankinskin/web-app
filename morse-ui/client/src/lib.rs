#![feature(async_closure)]
use components::{
    Component,
    Init,
    Viewable,
};
use seed::{
    prelude::*,
    *,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
    trace,
};
use std::{
    thread_local,
    time::{
        Duration,
    },
    rc::{
        Rc,
    },
};
use wasm_timer::{
    Instant,
    Delay,
};

use async_std::{
    stream::{
        Interval,
        StreamExt,
    },
    sync::{
        RwLock,
    },
};
mod slider;
use slider::*;
mod button;
use button::*;
mod audio;
use audio::*;

fn init_tracing() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
    debug!("Debug logs enabled.");
    info!("Info logs enabled.");
    trace!("Trace logs enabled.");
    error!("Error logs enabled.");
}
#[wasm_bindgen(start)]
pub async fn render() {
    init_tracing();
    debug!("Starting App");
    App::start(
            "app",
            Root::init,
            |msg, model, orders| model.update(msg, orders),
            Viewable::view,
        );
}

#[derive(Debug, Clone)]
enum Msg {
    Audio(AudioMsg),
    Slider(SliderMsg),
    Start,
    Stop,
    Rendered(RenderInfo),
}
struct Root {
    audio: Audio,   
    speed_slider: Slider,
    unit_time: Duration,
    units: f64,
    started: Option<f64>,
    times: Vec<(f64, f64)>,
    last_timestamp: Option<f64>,
    line_x: f64,
    ms_width: f64,
}
impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ButtonMsg| {
                match msg {
                    ButtonMsg::Click => Some(Msg::Start),
                    ButtonMsg::Leave | ButtonMsg::Release => Some(Msg::Stop),
                }
            })
            .after_next_render(Msg::Rendered);
        let unit_time = Duration::from_millis(50);
        let speed_slider = Slider::new(unit_time.as_millis() as f64, 1.0, 500.0, "ms per Unit");
        Self {
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
            speed_slider,
            unit_time,
            units: 0.,
            times: Vec::new(),
            started: None,
            last_timestamp: None,
            line_x: 280.0,
            ms_width: 0.05,
        }
    }
}
impl Root {
    pub fn millis_to_units(&self, timestamp: f64) -> f64 {
        timestamp/self.unit_time.as_millis() as f64
    }
    pub fn update_time(&mut self) {
        let timestamp = window().performance().unwrap().now();
        let delta = if let Some(prev) = self.last_timestamp.replace(timestamp) {
            timestamp - prev
        } else {
            timestamp
        };
        self.units += self.millis_to_units(delta);
    }
    pub fn unit_width(&self) -> f64 {
        self.ms_width * self.unit_time.as_millis() as f64
    }
    pub fn visible_units(&self) -> f64 {
        self.line_x/self.unit_width()
    }
    pub fn remove_invisible(&mut self) {
        let visible_units = self.visible_units();
        let units_now = self.units;
        self.times.retain(|(_, end)| *end >= units_now - visible_units);
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Audio(msg) => {
                self.audio.update(msg, &mut orders.proxy(Msg::Audio));
            },
            Msg::Slider(msg) => {
                match msg {
                    SliderMsg::Change(v) => {
                        self.update_time();
                        self.unit_time = Duration::from_millis(v as u64);
                    }
                }
                self.speed_slider.update(msg, &mut orders.proxy(Msg::Slider));
            },
            Self::Msg::Start => {
                debug!("Start!");
                self.update_time();
                self.started = Some(self.units);
                self.audio.start();
            },
            Self::Msg::Stop => {
                debug!("Stop!");
                if let Some(start) = self.started.take() {
                    self.update_time();
                    let end = self.units;
                    self.times.push((start, end));
                }
                self.audio.stop();
            },
            Self::Msg::Rendered(_) => {
                self.update_time();
                self.remove_invisible();
                orders.after_next_render(Msg::Rendered);
            }
        }
    }
}
impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        let (width, height) = (400, 100);
        let box_height = 5;
        let boxes: Vec<Node<Self::Msg>> = self.times.iter()
            .map(|(begin, end)| {
                let (w, h) = (self.unit_width() * (end - begin), box_height);
                let (x, y) = (self.line_x - self.unit_width() * (self.units-begin), 50);
                rect![
                    attrs!{
                        At::X => format!("{}px", x),
                        At::Y => format!("{}px", y),
                        At::Width => format!("{}px", w),
                        At::Height => format!("{}px", h),
                        At::Fill => "red",
                    }
                ]
            }).collect();
        let new_box: Node<Self::Msg> = if let Some(begin) = self.started {
            let (w, h) = (self.unit_width() * (self.units-begin), box_height);
            let (x, y) = (self.line_x - w, 50);
            rect![
                attrs!{
                    At::X => format!("{}px", x),
                    At::Y => format!("{}px", y),
                    At::Width => format!("{}px", w),
                    At::Height => format!("{}px", h),
                    At::Fill => "red",
                }
            ]
        } else {
            empty![]
        };
        let fract = self.visible_units().fract();
        let visible_units = self.visible_units().ceil() as u64;
        let guides: Vec<Node<Self::Msg>> = (0..visible_units).map(|unit| {
                    path![
                        attrs!{
                            At::D => format!("M{} 0 V{}", (self.line_x - fract) - unit as f64*self.unit_width() , height),
                            At::Stroke => "grey",
                        },
                    ]
                }).collect();
        div![
            "Morse",
            style!{
                St::UserSelect => "none"; 
            },
            p![format!("Boxes: {}", self.times.len())],
            svg![
                attrs!{
                    At::Width => format!("{}px", width),
                    At::Height => format!("{}px", height),
                },
                guides,
                path![
                    attrs!{
                        At::D => format!("M{} 0 V{}", self.line_x, height),
                        At::Stroke => "black",
                    },
                ],
                boxes,
                new_box,
                ev(Ev::MouseDown, |_| {
                    Self::Msg::Start       
                }),
                ev(Ev::MouseLeave, |_| {
                    Self::Msg::Stop
                }),
                ev(Ev::MouseUp, |_| {
                    Self::Msg::Stop       
                }),
            ],
            self.audio
                .view()
                .map_msg(Msg::Audio),
            self.speed_slider
                .view()
                .map_msg(Msg::Slider),
        ]
    }
}

