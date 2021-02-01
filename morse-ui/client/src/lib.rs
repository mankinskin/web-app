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
    Button(ButtonMsg),
    Audio(AudioMsg),
    Slider(SliderMsg),
    Start,
    Stop,
}
struct Root {
    button: Button,
    audio: Audio,   
    speed_slider: Slider,
    unit_time: Rc<RwLock<Duration>>,
}
impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ButtonMsg| {
            match msg {
                ButtonMsg::Click => Some(Msg::Start),
                ButtonMsg::Leave | ButtonMsg::Release => Some(Msg::Stop),
            }
        });
        let unit_time = Duration::from_millis(50);
        let speed_slider = Slider::new(unit_time.as_millis() as f64, 1.0, 500.0, "Speed");
        let unit_time = Rc::new(RwLock::new(unit_time));
        let unit_time2 = unit_time.clone();
        spawn_local(async move {
            let start = Instant::now();
            let unit_time = unit_time2;
            loop {
                let dur = unit_time.read().await.clone();
                async_std::task::sleep(dur).await;
                debug!("Tick {}", Instant::now().duration_since(start).as_millis());
            }
        });
        Self {
            button: Button,
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
            speed_slider,
            unit_time,
        }
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Button(msg) => {
                self.button.update(msg, &mut orders.proxy(Msg::Button));
            },
            Msg::Audio(msg) => {
                self.audio.update(msg, &mut orders.proxy(Msg::Audio));
            },
            Msg::Slider(msg) => {
                match msg {
                    SliderMsg::Change(v) => {
                        let unit_time = self.unit_time.clone();
                        spawn_local(async move {
                            *unit_time.write().await = Duration::from_millis(v as u64);
                        });
                    }
                }
                self.speed_slider.update(msg, &mut orders.proxy(Msg::Slider));
            },
            Self::Msg::Start => {
                debug!("Start!");
                self.audio.start();
            },
            Self::Msg::Stop => {
                debug!("Stop!");
                self.audio.stop();
            },
        }
    }
}
impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        div![
            "Hello World",
            self.button
                .view()
                .map_msg(Msg::Button),
            self.audio
                .view()
                .map_msg(Msg::Audio),
            self.speed_slider
                .view()
                .map_msg(Msg::Slider),
        ]
    }
}

