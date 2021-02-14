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
use std::time::Duration;
use web_sys::KeyboardEvent;
#[derive(Debug, Clone)]
pub enum Msg {
    Start,
    Stop,
    Rendered(RenderInfo),
}
pub struct Timeline {
    boxes: Vec<(f64, f64)>,
    guides: Vec<f64>,
    started: Option<f64>,
    unit_time: Duration,
    time_window: Duration,
    last_timestamp: Option<f64>,
    box_height: u32,
    width: u32,
    height: u32,
}
pub fn timestamp_now() -> Option<f64> {
    Some(window().performance()?.now())
}
impl Timeline {
    fn milli_width(&self) -> f64 {
        self.width as f64/self.time_window.as_millis() as f64
    }
    fn time_to_width(&self, time: f64) -> f64 {
        self.milli_width() * time
    }
    fn update_time(&mut self) {
        self.last_timestamp.replace(timestamp_now().unwrap());
    }
    fn take_timestamp(&mut self) -> f64 {
        if let Some(t) = self.last_timestamp {
            return t;
        }
        let t = timestamp_now().unwrap();
        self.last_timestamp = Some(t);
        t
    }
    fn remove_invisible(&mut self) {
        let now = self.take_timestamp();
        let window = self.time_window.as_millis() as f64;
        self.boxes.retain(|(_, end)| now - *end <= window);
        self.guides.retain(|t| now - *t <= window);
    }
    pub fn set_unit_time(&mut self, unit_time: Duration) {
        self.unit_time = unit_time;
        self.update_time();
    }
    fn start(&mut self) {
        if self.started.is_none() {
            let now = self.take_timestamp();
            self.started = Some(now);
        }
    }
    fn stop(&mut self) {
        if let Some(start) = self.started.take() {
            let now = self.take_timestamp();
            self.boxes.push((start, now));
        }
    }
    fn boxes(&self) -> Vec<Node<Msg>> {
        if let Some(now) = self.last_timestamp {
            self.boxes.iter()
                .map(|(begin, end)| {
                    let (w, h) = (self.time_to_width(end - begin), self.box_height);
                    let (x, y) = (self.width as f64 - self.time_to_width(now-begin), self.height/2);
                    rect![
                        attrs!{
                            At::X => format!("{}px", x),
                            At::Y => format!("{}px", y),
                            At::Width => format!("{}px", w),
                            At::Height => format!("{}px", h),
                            At::Fill => "red",
                        }
                    ]
                }).collect()
        } else {
            Vec::new()
        }
    }
    fn new_box(&self) -> Node<Msg> {
        if let Some((now, begin)) = self.last_timestamp.zip(self.started) {
            let (w, h) = (self.time_to_width(now-begin), self.box_height);
            let (x, y) = (self.width as f64 - w, self.height/2);
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
        }
    }
    fn start_line(&self) -> Node<Msg> {
        path![
            attrs!{
                At::D => format!("M{} 0 V{}", self.width, self.height),
                At::Stroke => "black",
            },
        ]
    }
    fn guides(&self) -> Vec<Node<Msg>> {
        if let Some(now) = self.last_timestamp {
            self.guides.iter().map(|t| {
                path![
                    attrs!{
                        At::D => format!("M{} 0 V{}", self.width as f64 - self.time_to_width(now - t), self.height),
                        At::Stroke => "grey",
                    },
                ]
            }).collect()
        } else {
            Vec::new()
        }
    }
}
impl Init<Duration> for Timeline {
    fn init(unit_time: Duration, orders: &mut impl Orders<Msg>) -> Self {
        orders
            .stream(streams::window_event(Ev::KeyDown, |event| {
                let event: KeyboardEvent = event.unchecked_into();
                debug!("{}", event.key_code());
                match event.key_code() {
                    32 => Some(Msg::Start),
                    _ => None
                }
            }))
            .stream(streams::window_event(Ev::KeyUp, |event| {
                let event: KeyboardEvent = event.unchecked_into();
                debug!("{}", event.key_code());
                match event.key_code() {
                    32 => Some(Msg::Stop),
                    _ => None
                }
            }))
            .after_next_render(Msg::Rendered);
        Self {
            boxes: Vec::new(),
            started: None,
            unit_time,
            time_window: Duration::from_secs(3),
            last_timestamp: None,
            box_height: 5,
            width: 400, 
            height: 100,
            guides: Vec::new(),
        }
    }
}
impl Component for Timeline {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        self.update_time();
        match msg {
            Self::Msg::Start => {
                //debug!("Start!");
                self.start()
            },
            Self::Msg::Stop => {
                //debug!("Stop!");
                self.stop()
            },
            Self::Msg::Rendered(_) => {
                self.remove_invisible();
                orders.after_next_render(Msg::Rendered);
            }
        }
        if let Some(start) = self.boxes.first().map(|(s, _)| *s).or(self.started) {
            if let Some(last) = self.guides.last().cloned() {
                let now = self.take_timestamp();
                let unit_time = self.unit_time.as_millis() as f64;
                let n = ((now - last)/unit_time).floor() as u32;
                self.guides.extend((1..=n).map(|k| last + k as f64 * unit_time));
            } else {
                self.guides.push(start);
            }
        }
    }
}
impl Viewable for Timeline {
    fn view(&self) -> Node<Msg> {
        div![
            p![format!("Boxes: {}", self.boxes.len())],
            p![format!("Guides: {}", self.guides.len())],
            svg![
                attrs!{
                    At::Width => format!("{}px", self.width),
                    At::Height => format!("{}px", self.height),
                },
                self.guides(),
                self.start_line(),
                self.boxes(),
                self.new_box(),
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
