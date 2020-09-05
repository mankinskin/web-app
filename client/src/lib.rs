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
extern crate openlimits;
extern crate rust_decimal;

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
use openlimits::{
    model::{
        Candle,
    },
};
use rust_decimal::prelude::ToPrimitive;

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start("app",
                |_url, orders| {
                    orders.send_msg(Msg::GetHistory); 
                    Model::default()
               },
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Clone)]
pub struct Model {
    width: u32,
    height: u32,
    data: Vec<Candle>,
    error: Option<String>,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            width: 500,
            height: 200,
            data: Vec::new(),
            error: None,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetHistory,
    GotHistory(Result<Vec<Candle>, String>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::GetHistory => {
                orders.perform_cmd(
                        async {
                            Self::fetch_candles()
                                .await
                                .map_err(|e| format!("{:#?}", e))
                        }
                        .map(Msg::GotHistory)
                    );
            },
            Msg::GotHistory(res) => {
                //log!(res);
                match res {
                    Ok(candles) => self.data = candles,
                    Err(e) => self.error = Some(e),
                }
            },
        }
    }
}
impl Model {
    async fn fetch_candles() -> Result<Vec<Candle>, FetchError> {
        let host = "http://localhost:8000";
        let url = format!("{}{}", host, "/api/price_history");
        let mut req = seed::fetch::Request::new(&url)
            .method(Method::Get);
        seed::fetch::fetch(req)
            .await?
            .check_status()?
            .json()
            .await
    }
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
    fn horizontal_lines(&self) -> Vec<Node<<Self as Component>::Msg>> {
            (0..(self.height/10)).fold(Vec::with_capacity(self.height as usize/10*2), |mut acc, index| {
                let y = index*10;
                let x = 0;
                let l = self.width as i32;
                let xp = 10;
                let yp = 10;

                let center_dir = (self.width as i32/2 - x).clamp(-1, 1);
                let emphathize = index % 10 == 0;
                let opacity = if emphathize { 0.3 } else { 0.1 };
                acc.push(path![
                        attrs!{
                            At::D => format!("M {} {} H {}", x, y, x + center_dir * l),
                            At::Stroke => "black",
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
                        ((self.height - y)/10).to_string()
                    ]);
                }
                acc
            })
    }
    fn update_button(&self) -> Node<<Self as Component>::Msg> {
        div![
            button![
                ev(Ev::Click, |_| Msg::GetHistory),
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
    fn graph_view(&self) -> Node<<Self as Component>::Msg> {
        let points: Vec<(_, _)> =
                self.data
                    .iter()
                    .enumerate()
                    .map(|(i, candle)|
                        (i, candle.open
                              .to_f32()
                              .unwrap()*1000000 as f32)
                    )
                    .collect();
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
                    At::ViewBox => format!("0 0 {} {}", self.width, self.height),
                    At::PreserveAspectRatio => "xMinYMin meet",
                },
                text![
                    attrs!{
                        At::X => self.width/2,
                        At::Y => self.height/2,
                        At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                        At::DominantBaseline => "middle",
                        At::FontSize => "18",
                        At::Fill => "black",
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
                self.vertical_lines(),
                self.horizontal_lines(),
            ],
        ]
    }
}
// Graph SVG
// viewport (history_length+pad, max_price+pad)
// viewbox scalable
//
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            self.update_button(),
            self.graph_view(),
        ]
    }
}
