use components::{
    Component,
    Init,
    Viewable,
};
use rust_decimal::prelude::ToPrimitive;
use seed::{
    prelude::*,
    *,
};
use tracing::{
    debug,
};
use openlimits::model::{
    Candle,
};
#[derive(Debug)]
pub struct Chart {
    pub svg_node: Option<web_sys::Node>,
    pub data: Vec<Candle>,
    pub data_min: f32,
    pub data_max: f32,
    pub y_interval: u32,
    pub y_factor: f32,
    pub data_range: f32,
    pub x_interval: u32,
    pub view_x: i32,
    pub view_y: i32,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug)]
pub enum Msg {
}
impl Init<()> for Chart {
    fn init(_: (), _orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        debug!("Creating chart");
        Self {
            view_x: 0,
            view_y: 0,
            width: 800,
            height: 500,
            data: Vec::new(),
            data_min: 0.0,
            data_max: 0.0,
            x_interval: 2,
            y_interval: 0,
            y_factor: 0.0,
            svg_node: None,
            data_range: 0.0,
        }
    }
}
impl Component for Chart {
    type Msg = Msg;
    fn update(&mut self, _msg: Msg, _orders: &mut impl Orders<Msg>) {
        //debug!("Chart update");
    }
}
impl Chart {
    pub fn clear(&mut self) {
        self.data.clear();
    }
    pub fn append_data<I: IntoIterator<Item=Candle>>(&mut self, new_data: I) {
        self.data.extend(new_data);
    }
    pub fn set_data<I: IntoIterator<Item=Candle>>(&mut self, new_data: I) {
        self.data = new_data.into_iter().collect();
    }
    pub fn update_values(&mut self) {
        self.data_max = self
            .data
            .iter()
            .map(|candle| candle.high)
            .max()
            .map(|d| d.to_f32().unwrap())
            .unwrap_or(0.0);
        self.data_min = self
            .data
            .iter()
            .map(|candle| candle.low)
            .min()
            .map(|d| d.to_f32().unwrap())
            .unwrap_or(0.0);
        self.data_range = self.data_max - self.data_min;
        if self.data_range != 0.0 {
            self.y_factor = self.height as f32 / self.data_range;
        }
        self.y_interval = (0.000001 * self.y_factor).round() as u32;
        //log!(self)
    }
    fn to_y_pixels(&self, d: f32) -> i32 {
        (d * self.y_factor).round() as i32
    }
    fn graph_view(&self) -> Node<<Self as Component>::Msg> {
        div![
            style! {
                St::Overflow => "scroll",
                //St::OverflowY => "auto",
                //St::Height => "auto",
                //St::Cursor => "move",
                //St::Resize => "horizontal",
            },
            //ev(Ev::Click, |event| {
            //    event.prevent_default();
            //    let event = to_mouse_event(&event);
            //    let x = event.movement_x();
            //    let y = event.movement_y();
            //    log!("Panning {} {}", x, y);
            //    Msg::Panning(x, y)
            //}),
            svg![
                attrs! {
                    At::ViewBox => format!("{} {} {} {}", self.view_x, self.view_y, self.width, self.height),
                    At::PreserveAspectRatio => "xMinYMin meet",
                    At::Width => self.width,
                    At::Height => self.height,
                    At::Id => "graph-svg",
                    At::Overflow => "scroll",
                },
                style! {
                    St::BackgroundColor => "gray";
                    St::Width => "100%",
                },
                self.plot_view(),
                self.vertical_lines(),
                self.horizontal_lines(),
            ],
        ]
    }
    fn plot_view(&self) -> Vec<Node<<Self as Component>::Msg>> {
        let candles: Vec<(_, _)> = self.data.iter().enumerate().collect();
        candles.iter().fold(
            Vec::with_capacity(self.data.len() * 2),
            |mut acc, (i, candle)| {
                let open = candle.open.to_f32().unwrap();
                let close = candle.close.to_f32().unwrap();
                let high = candle.high.to_f32().unwrap();
                let low = candle.low.to_f32().unwrap();
                let height = self.to_y_pixels((open - close).abs());
                let top = open.max(close);
                let x_px = *i as u32 * self.x_interval;
                let y_px = self.height - self.to_y_pixels(top - self.data_min) as u32;
                let color = if open > close {
                    "red"
                } else if open == close {
                    "gray"
                } else {
                    "green"
                };
                acc.push(rect![attrs! {
                    At::X => x_px,
                    At::Y => y_px,
                    At::Width => self.x_interval,
                    At::Height => height,
                    At::Fill => color,
                },]);
                let x_px = x_px + self.x_interval / 2;
                let y_px = self.height - self.to_y_pixels(high - self.data_min) as u32;
                let height = self.to_y_pixels(high - low);
                acc.push(path![attrs! {
                    At::D => format!("M {} {} v {}", x_px, y_px, height),
                    At::Stroke => color,
                    At::StrokeWidth => 1,
                },]);
                acc
            },
        )
    }
    #[allow(unused)]
    fn vertical_lines(&self) -> Vec<Node<<Self as Component>::Msg>> {
        (0..(self.width / 10)).fold(
            Vec::with_capacity(self.width as usize / 10 * 2),
            |mut acc, index| {
                let x = index * 10;
                let y = self.height as i32;
                let h = 10;
                let padding = 10;

                let center_dir = (-1 * self.height as i32 / 2).clamp(-1, 1);
                acc.push(path![attrs! {
                    At::D => format!("M {} {} V {}", x, y, y + center_dir * h)
                    At::Stroke => "black"
                }]);
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
            },
        )
    }
    #[allow(unused)]
    fn horizontal_lines(&self) -> Vec<Node<<Self as Component>::Msg>> {
        let count: usize = self.height as usize / self.y_interval.max(1) as usize;
        (0..count).fold(Vec::with_capacity(count * 2), |mut acc, index| {
            let y: i32 = self.height as i32 - index as i32 * self.y_interval as i32;
            let x: i32 = 0;
            let l = self.width as i32;
            let xp: i32 = 10;
            let yp: i32 = 10;

            let center_dir = (self.width as i32 / 2 - x).clamp(-1, 1);
            let emphathize = index % 10 == 0;
            let opacity = if emphathize { 0.3 } else { 0.1 };
            acc.push(path![attrs! {
                At::D => format!("M {} {} h {}", x, y, center_dir * l),
                At::Stroke => "black",
                At::StrokeWidth => 1,
                At::Opacity => opacity,
            }]);
            if emphathize {
                acc.push(text![
                    attrs! {
                        At::X => x + xp,
                        At::Y => y + yp,
                        At::FontFamily => "-apple-system, system-ui, BlinkMacSystemFont, Roboto",
                        At::DominantBaseline => "middle",
                        At::TextAnchor => "middle",
                        At::FontSize => "9",
                        At::Fill => "black",
                    },
                    ((self.height as i32 - y) / 10).to_string()
                ]);
            }
            acc
        })
    }
}
impl Viewable for Chart {
    fn view(&self) -> Node<Self::Msg> {
        //debug!("Chart redraw!");
        div![
            self.graph_view(),
        ]
    }
}
