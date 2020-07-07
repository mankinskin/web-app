use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
    },
    root::{
        GMsg,
    },
};

#[derive(Clone, Default)]
pub struct Model {
}
#[derive(Clone)]
pub enum Msg {
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, _msg: Self::Msg, _orders: &mut impl Orders<Self::Msg, GMsg>) {
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
        ]
    }
}
