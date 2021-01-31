use components::{
    Component,
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
#[derive(Debug, Clone)]
pub enum SliderMsg {
    Change(f64),
}
#[derive(Clone)]
pub struct Slider {
    label: String,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
}
impl Slider {
    pub fn new(value: f64, min: f64, max: f64, label: impl ToString) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            step: 0.00001,
        }
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn set_value(&mut self, v: f64) {
        self.value = v;
    }
}
impl Component for Slider {
    type Msg = SliderMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match &msg {
            Self::Msg::Change(v) => self.set_value(*v),
        }
        orders.notify(msg);
    }
}
impl Viewable for Slider {
    fn view(&self) -> Node<SliderMsg> {
        div![
            &self.label,
            input![
                attrs!{
                    At::Type => "range";
                    At::Min => format!("{}", self.min);
                    At::Max => format!("{}", self.max);
                    At::Value => format!("{}", self.value);
                    At::Step => format!("{}", self.step);
                },
                input_ev(Ev::Input, |s| {
                    SliderMsg::Change(s.parse::<f64>().expect("Failed to parse slider value to f64"))
                })
            ],
            format!("{}", self.value),
        ]
    }
}
