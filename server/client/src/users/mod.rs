use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use crate::{
    root::{
        GMsg,
    },
};
use database::{
    Entry,
};

pub mod preview;
pub mod profile;
pub mod user;

#[derive(Clone)]
pub struct Model {
    users: Vec<Entry<User>>,
    previews: Vec<preview::Model>,
}
#[derive(Clone)]
pub enum Config {
    Empty,
    All,
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::Empty => Model::empty(),
        Config::All => { orders.send_msg(Msg::GetAll); Model::empty() },
    }
}
impl Model {
    pub fn empty() -> Self {
        Self {
            users: vec![],
            previews: vec![],
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Msg {
    GetAll,
    AllUsers(Result<Vec<Entry<User>>, String>),
    Preview(usize, preview::Msg),
}
impl Msg {
    pub fn fetch_users() -> Msg {
        Msg::GetAll
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::AllUsers(res) => {
            match res {
                Ok(ps) => model.previews = ps.iter().map(|u| preview::Model::from(u)).collect(),
                Err(e) => { seed::log(e); },
            }
        },
        Msg::GetAll => {
            orders.perform_cmd(
                api::get_users()
                    .map(|res| Msg::AllUsers(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::Preview(index, msg) => {
            preview::update(
                msg,
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        ul![
            model.previews.iter().enumerate()
                .map(|(i, preview)| li![
                     preview::view(preview)
                        .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                ])
        ]
    ]
}
