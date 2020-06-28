use seed::{
    *,
    prelude::*,
};
use crate::{
    root::{
        GMsg,
    },
};

#[derive(Clone, Default)]
pub struct Model {
}
impl Model {
    pub fn empty() -> Self {
        Model::default()
    }
}
#[derive(Clone, Default)]
pub struct Config {
}
#[derive(Clone)]
pub enum Msg {
}
pub fn init(_config: Config, _orders: &mut impl Orders<Msg, GMsg>) -> Model {
    Model::empty()
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
