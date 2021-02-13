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
    time::{
        Duration,
    },
};
mod slider;
use slider::*;
mod button;
mod audio;
use audio::*;
mod timeline;
use timeline::*;

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
    Timeline(timeline::Msg),
}
struct Root {
    audio: Audio,   
    speed_slider: Slider,
    timeline: Timeline,
}
impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        let unit_time = Duration::from_millis(50);
        Self {
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
            speed_slider: Slider::new(unit_time.as_millis() as f64, 1.0, 500.0, "ms per Unit"),
            timeline: Timeline::init(unit_time, &mut orders.proxy(Msg::Timeline)),
        }
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
                        self.timeline.set_unit_time(Duration::from_millis(v as u64));
                    }
                }
                self.speed_slider.update(msg, &mut orders.proxy(Msg::Slider));
            },
            Msg::Timeline(msg) => {
                match msg {
                    timeline::Msg::Start => self.audio.start(),
                    timeline::Msg::Stop => self.audio.stop(),
                    _ => {}
                }
                self.timeline.update(msg, &mut orders.proxy(Msg::Timeline));
            },
        }
    }
}

impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        div![
            "Morse",
            style!{
                St::UserSelect => "none"; 
            },
            self.timeline
                .view()
                .map_msg(Msg::Timeline),
            self.audio
                .view()
                .map_msg(Msg::Audio),
            self.speed_slider
                .view()
                .map_msg(Msg::Slider),
        ]
    }
}

