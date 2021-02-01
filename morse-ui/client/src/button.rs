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
pub enum ButtonMsg {
    Click,
    Release,
    Leave,
}
pub struct Button;

impl Component for Button {
    type Msg = ButtonMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match &msg {
            Self::Msg::Click => {},
            Self::Msg::Release => {},
            Self::Msg::Leave => {},
        }
        orders.notify(msg);
    }
}
impl Viewable for Button {
    fn view(&self) -> Node<<Self as Component>::Msg> {
        button![
            "Click!",
            ev(Ev::MouseDown, |_| {
                Self::Msg::Click       
            }),
            ev(Ev::MouseLeave, |_| {
                Self::Msg::Leave
            }),
            ev(Ev::MouseUp, |_| {
                Self::Msg::Release       
            }),
        ]
    }
}
