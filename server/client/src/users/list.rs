use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use crate::{
    root,
    config::*,
    users::{
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
impl Component for Model {
    type Msg = Msg;
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
//impl Config<Model> for Vec<Entry<User>> {
//    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
//        Model::from(self)
//    }
//    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
//        orders.send_msg(self);
//    }
//}
impl From<Vec<Entry<User>>> for Model {
    fn from(entries: Vec<Entry<User>>) -> Self {
        Self {
            previews: init_previews(entries),
        }
    }
}
//impl From<Vec<Entry<User>>> for Msg {
//    fn from(entry: Vec<Entry<User>>) -> Self {
//        Msg::None
//    }
//}
#[derive(Clone)]
pub enum Msg {
    Preview(usize, preview::Msg),

    GetAll,
    AllUsers(Result<Vec<Entry<User>>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Preview(index, msg) => {
            preview::update(
                msg,
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
            );
        },
        Msg::GetAll => {
            orders.perform_cmd(
                api::get_users()
                    .map(|res| Msg::AllUsers(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::AllUsers(res) => {
            match res {
                Ok(users) => model.previews = init_previews(users),
                Err(e) => { seed::log(e); },
            }
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
