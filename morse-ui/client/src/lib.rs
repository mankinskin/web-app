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
use std::time::Duration;
use web_sys::{
    Element,
    HtmlElement,
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
#[allow(unused)]
fn document_body() -> Option<HtmlElement> {
    window().document()?.body()
}
fn document_element() -> Option<Element> {
    window().document()?.document_element()
}
fn init_document() {
    let document = document_element().expect("Failed to get document element");
    let old_styles = document.get_attribute("style").unwrap_or("".into());
    let new_styles = vec![
        (St::BackgroundColor, "#bcb6b3"),
        (St::FontSize, "12pt"),
    ].iter().fold(old_styles,
        |acc, (st, v)| format!("{};{}: {}", acc, st, v));
    document.set_attribute("style", &new_styles)
        .expect("Failed to set document attribute");
}
#[wasm_bindgen(start)]
pub async fn render() {
    init_tracing();
    //debug!("Starting App");
    init_document();
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
        let unit_time = Duration::from_millis(150);
        Self {
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
            speed_slider: Slider::new(unit_time.as_millis() as f64, 50.0, 500.0, "ms per Unit"),
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
            style!{
                St::Height => "60%";
                St::Display => "flex";
                St::AlignItems => "center";
            },
            div![
                style!{
                    St::UserSelect => "none"; 
                    St::Margin => "auto";
                    St::MinWidth => "60%";
                    St::MaxWidth => "90%";
                    St::BackgroundColor => "#f2efdc";
                    St::Padding => "5px 20px 20px 20px";
                    St::BorderRadius => "20px";
                    St::FontFamily => "Courier New";
                },
                h1![
                    style!{
                        St::TextAlign => "center";
                    },
                    "Morse Code Reader"
                ],
                self.timeline
                    .view()
                    .map_msg(Msg::Timeline),
                p![
                    style!{
                        St::FontSize => "9pt";
                    },
                    "Parsed Text"
                ],
                p![
                    style!{
                        St::MinHeight => "12pt";
                        St::FontSize => "9pt";
                        St::BackgroundColor => "#ffffff";
                        St::BorderRadius => "5px";
                        St::Padding => "3px";
                    },
                    "text...",
                ],
                self.audio
                    .view()
                    .map_msg(Msg::Audio),
                self.speed_slider
                    .view()
                    .map_msg(Msg::Slider),
            ]
        ]
    }
}

