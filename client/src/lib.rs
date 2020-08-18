#![feature(clamp)]
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
extern crate rand;
extern crate rand_distr;

use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    View,
};
use rand::{
    prelude::*,
};
use rand_distr::{
    Normal,
};

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start("app",
               |_url, _orders| Model::default(),
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Clone, Default)]
pub struct Model {
    width: usize,
    height: usize,
}
#[derive(Clone, Debug)]
pub enum Msg {
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        let width = 1000;
        let height = 400;
        let mut rng = rand::thread_rng();
        let points: Vec<(_, _)> = (0..width)
            .zip(Normal::new(height as f32/2.0, 20.0).unwrap().sample_iter(&mut rng).take(width))
            .collect();

        div![
            p!["Hello from Seed!"],
            div![
                style!{
                    St::Resize => "horizontal",
                    St::Overflow => "auto",
                    St::Height => "auto",
                },
                svg![
                    style!{
                        St::BackgroundColor => "gray";
                    },
                    attrs!{
                        At::ViewBox => format!("0 0 {} {}", width, height),
                        At::PreserveAspectRatio => "xMinYMin meet",
                    },
                    text![
                        attrs!{
                            At::X => width/2,
                            At::Y => height/2,
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
                                        At::D => format!("M {} {} L {} {}", ax, ay, bx, by)
                                        At::Stroke => "black"
                                    }
                                ],
                                circle![
                                    attrs!{
                                        At::Cx => format!("{}", bx)
                                        At::Cy => format!("{}", by)
                                        At::R => "1",
                                        At::Fill => "red",
                                    }
                                ],
                            ]
                        } else {
                            vec![]
                        }
                    }),
                    (0..(width/10)).map(|index| {
                        let x = index*10;
                        let y = height;
                        let h = 10;
                        let padding = 10;

                        let center_dir = (height as i32/2 - y).clamp(-1, 1);
                        let mut elems = Vec::with_capacity(2);
                        elems.push(path![
                                attrs!{
                                    At::D => format!("M {} {} V {}", x, y, y + center_dir * h)
                                    At::Stroke => "black"
                                }
                            ]);
                        if index % 10 == 0 {
                            elems.push(text![
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
                        elems
                    }),
                    (0..(height/10)).map(|index| {
                        let y = index*10;
                        let x = 0;
                        let l = width as i32;
                        let xp = 10;
                        let yp = 10;

                        let center_dir = (width as i32/2 - x).clamp(-1, 1);
                        let mut elems = Vec::with_capacity(2);
                        let emphathize = index % 10 == 0;
                        let opacity = if emphathize { 0.3 } else { 0.1 };
                        elems.push(path![
                                attrs!{
                                    At::D => format!("M {} {} H {}", x, y, x + center_dir * l),
                                    At::Stroke => "black",
                                    At::Opacity => opacity,
                                }
                            ]);
                        if emphathize {
                            elems.push(text![
                                attrs!{
                                    At::X => x + xp,
                                    At::Y => y + yp,
                                    At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                                    At::DominantBaseline => "middle",
                                    At::TextAnchor => "middle",
                                    At::FontSize => "9",
                                    At::Fill => "black",
                                },
                                ((height - y)/10).to_string()
                            ])
                        }
                        elems
                    })
                ]
            ]
        ]
    }
}
