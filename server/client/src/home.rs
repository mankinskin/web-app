use seed::{
    *,
    prelude::*,
};

#[derive(Clone, Default)]
pub struct Model {
}

#[derive(Clone)]
pub enum Msg {
}

pub fn update(_msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg>) {
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
