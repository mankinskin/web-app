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
mod slider;
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
pub fn render() {
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
    Click,
    Release
}
struct Root {
    button: Button,
    audio: Audio,   
}
impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ButtonMsg| {
            match msg {
                ButtonMsg::Click => Some(Msg::Click),
                ButtonMsg::Release => Some(Msg::Release),
            }
        });
        Self {
            button: Button,
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
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
            Self::Msg::Click => {
                debug!("Click!");
                self.audio.start();
            },
            Self::Msg::Release => {
                debug!("Release!");
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
        ]
    }
}

