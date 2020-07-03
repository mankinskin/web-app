use seed::{
    *,
    prelude::*,
};
use crate::{
    config::*,
    root::{
        GMsg,
    },
};

#[derive(Clone, Default)]
pub struct Model {
}
impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone)]
pub enum Msg {
}
pub fn update(_msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg, GMsg>) {
}
pub fn view(_model: &Model) -> Node<Msg> {
    ul![
        li![
            "Awesome Stuff"
        ],
        li![
            "Look at this too!"
        ],
    ]
}
