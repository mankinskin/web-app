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
    time::{
        Duration,
    },
};
#[derive(Debug, Clone)]
pub enum Msg {
    Start,
    Stop,
    Rendered(RenderInfo),
}
pub struct Timeline {
    times: Vec<(f64, f64)>,
    line_x: f64,
    ms_width: f64,
    started: Option<f64>,
    unit_time: Duration,
    last_timestamp: Option<f64>,
    units: f64,
}
pub fn timestamp_now() -> Option<f64> {
    Some(window().performance()?.now())
}
impl Timeline {
    pub fn millis_to_units(&self, timestamp: f64) -> f64 {
        timestamp/self.unit_time.as_millis() as f64
    }
    pub fn update_time(&mut self) {
        let timestamp = timestamp_now().unwrap();
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
    pub fn set_unit_time(&mut self, unit_time: Duration) {
        self.unit_time = unit_time;
        self.update_time();
    }
}
impl Init<Duration> for Timeline {
    fn init(unit_time: Duration, orders: &mut impl Orders<Msg>) -> Self {
        orders.after_next_render(Msg::Rendered);
        Self {
            times: Vec::new(),
            line_x: 280.0,
            ms_width: 0.05,
            started: None,
            unit_time,
            units: 0.,
            last_timestamp: None,
        }
    }
}
impl Component for Timeline {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        self.update_time();
        match msg {
            Self::Msg::Start => {
                debug!("Start!");
                self.started = Some(self.units);
            },
            Self::Msg::Stop => {
                debug!("Stop!");
                if let Some(start) = self.started.take() {
                    let end = self.units;
                    self.times.push((start, end));
                }
            },
            Self::Msg::Rendered(_) => {
                self.remove_invisible();
                orders.after_next_render(Msg::Rendered);
            }
        }
    }
}
impl Viewable for Timeline {
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
        ]
    }
}
