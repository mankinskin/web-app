use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use crate::{
    root,
    config::{
        Config,
        Component,
        View,
    },
    user::{
        preview,
    },
    root::{
        GMsg,
    },
};
use database::{
    Entry,
};

fn init_previews(entries: Vec<Entry<User>>) -> Vec<preview::Model> {
    entries.iter().map(|u| preview::Model::from(u.clone())).collect()
}

#[derive(Clone, Default)]
pub struct Model {
    previews: Vec<preview::Model>,
}
impl Config<Model> for list::Msg<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::List(self));
    }
}
impl Config<Model> for Msg {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        match self {
            _ => Model::default(),
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(self);
    }
}
impl From<Vec<Entry<User>>> for Model {
    fn from(entries: Vec<Entry<User>>) -> Self {
        Self {
            previews: init_previews(entries),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Preview(usize, preview::Msg),

    GetAll,
    AllUsers(Result<Vec<Entry<User>>, String>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Preview(index, msg) => {
                self.previews[index].update(
                    msg,
                    &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
                );
            },
            Msg::GetAll => {
                orders.perform_cmd(
                    api::get_user()
                        .map(|res| Msg::AllUsers(res.map_err(|e| format!("{:?}", e))))
                );
            },
            Msg::AllUsers(res) => {
                match res {
                    Ok(user) => self.previews = init_previews(user),
                    Err(e) => { seed::log(e); },
                }
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            ul![
                self.previews.iter().enumerate()
                    .map(|(i, preview)| li![
                         preview.view()
                            .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                    ])
            ]
        ]
    }
}
