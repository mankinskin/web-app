use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
    },
};

#[derive(Debug,Clone, Default)]
pub struct Model {
    name: String,
}
#[derive(Clone, Debug)]
pub enum Msg {
    SetName(String),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, _orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::SetName(n) => {
                log!(n)
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        ul![
            li![
                "Awesome Stuff"
            ],
            li![
                "Look at this too!"
            ],
            input![
                attrs!{
                    At::Placeholder => "Name",
                    At::Value => self.name,
                },
                input_ev(Ev::Input, Msg::SetName)
            ],
        ]
    }
}
