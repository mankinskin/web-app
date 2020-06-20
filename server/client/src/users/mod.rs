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
    fetch::{
        self,
        Request,
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
    users: Vec<Entry<User>>,
    previews: Vec<preview::Model>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            users: vec![],
            previews: vec![],
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetch::Msg<Vec<Entry<User>>>),
    Preview(usize, preview::Msg),
}
impl Msg {
    pub fn fetch_users() -> Msg {
        Msg::Fetch(fetch::Msg::Request(Request::Get(Query::All)))
    }
}
impl From<fetch::Msg<Vec<Entry<User>>>> for Msg {
    fn from(msg: fetch::Msg<Vec<Entry<User>>>) -> Self {
        Msg::Fetch(msg)
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/users").unwrap(),
                            request,
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        fetch::Response::Get(data) => {
                            model.previews = data.iter().map(|u| preview::Model::from(u)).collect()
                        },
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {

                },
            }
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
