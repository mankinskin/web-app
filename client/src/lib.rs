extern crate async_tls;
extern crate lazy_static;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate seed;
extern crate console_error_panic_hook;
extern crate components;
use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    Config,
    View,
};

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start("app",
               |url, orders| Model::default(),
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Clone, Default)]
pub struct Model {
}
#[derive(Clone, Debug)]
pub enum Msg {
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        let width = 100;
        let height = 100;
        let points: Vec<(_, _)> = (0..10)
            .zip(vec![10, 20, 25, 13, 26, 17, 18, 15, 20, 19])
            .collect();
        div![
            p!["Hello from Seed!"],
            svg![
                attrs!{
                    At::Width => "50%",
                    At::Height => "50%",
                    At::ViewBox => format!("0 0 {} {}", width, height),
                    At::PreserveAspectRatio => "xMidYMid meet",
                },
                text![
                    attrs!{
                        At::X => (width/2).to_string(),
                        At::Y => (height/2).to_string(),
                        At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                        At::DominantBaseline => "middle",
                        At::TextAnchor => "middle",
                        At::FontSize => "18",
                        At::Fill => "#74838f",
                        At::FontWeight => "700",
                    },
                    "Example SVG"
                ],
                points.windows(2).map(|window| {
                    if let [(ax, ay), (bx, by)] = window {
                        vec![
                            path![
                                attrs!{
                                    At::D => format!("M {} {} L {} {}", ax*10, ay, bx*10, by)
                                    At::Stroke => "black"
                                }
                            ],
                            circle![
                                attrs!{
                                    At::Cx => format!("{}", bx*10)
                                    At::Cy => format!("{}", by)
                                    At::R => "1",
                                    At::Fill => "black",
                                    At::Stroke => "red",
                                }
                            ],
                        ]
                    } else {
                        vec![]
                    }
                })
            ]
        ]
    }
}
