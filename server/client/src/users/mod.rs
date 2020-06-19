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
    fetched::{
        self,
        Status,
        Fetched,
        Query,
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
    users: Fetched<Vec<Entry<User>>>,
    previews: Vec<preview::Model>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            users:
                Fetched::empty(
                       url::Url::parse("http://localhost:8000/api/users").unwrap(),
                       Query::all()
                ),
            previews: vec![],
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    FetchUsers(fetched::Msg<Vec<Entry<User>>>),
    UserPreview(usize, preview::Msg),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::FetchUsers(msg) => {
            model.users.update(msg, &mut orders.proxy(Msg::FetchUsers));
            if let Status::Ready(users) = model.users.status() {
                model.previews = users.iter().map(|u| preview::Model::from(u)).collect()
            }
        },
        Msg::UserPreview(index, msg) => {
            preview::update(
                msg,
                &mut model.previews[index],
                &mut orders.proxy(move |msg| Msg::UserPreview(index.clone(), msg))
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.users.status() {
        Status::Ready(users) => {
            div![
                ul![
                    users.iter().enumerate()
                        .map(|(i, entry)| li![
                             preview::view(&preview::Model::from(entry))
                                .map_msg(move |msg| Msg::UserPreview(i.clone(), msg))
                        ])
                ]
            ]
        },
        Status::Waiting => {
            div![
                format!("Fetching...")
            ]
        },
        Status::Empty => {
            div![
                format!("Empty...")
            ]
        },
        Status::Failed(s) => {
            div![
                format!("Failed: {}", s)
            ]
        },
    }
}
