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

#[derive(Clone, Default)]
pub struct Model {
}
#[derive(Clone)]
pub enum Msg {
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, _msg: Self::Msg, _orders: &mut impl Orders<Self::Msg>) {
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
