use seed::{
    *,
    prelude::*,
};
use futures::{
    Future,
};
use plans::{
    user::*,
};
use crate::{
    root::{
        self,
        GMsg,
    },
    status::{
        Status,
    },
};
use rql::{
    Id,
};
use database::{
    Entry,
};
use std::result::Result;

pub mod preview;
pub mod profile;
pub mod user;

#[derive(Clone)]
pub struct Model {
    users: Status<Vec<preview::Model>>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            users: Status::Empty,
        }
    }
}
impl From<Vec<Id<User>>> for Model {
    fn from(users: Vec<Id<User>>) -> Self {
        Self {
            users: Status::Ready(users
                .iter()
                .map(|id| user::Model::from(*id).preview())
                .collect()),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    FetchUsers,
    FetchedUsers(ResponseDataResult<Vec<Entry<User>>>),
    UserPreview(usize, preview::Msg),
}
fn fetch_all_users(session: UserSession)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/api/users")
        .header("authorization", &format!("{}", session.token))
        .method(Method::Get)
            .fetch_json_data(move |data_result: ResponseDataResult<Vec<Entry<User>>>| {
                Msg::FetchedUsers(data_result)
            })
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::FetchUsers => {
            if let Some(session) = root::get_session() {
                orders.perform_cmd(fetch_all_users(session));
                model.users = Status::Loading;
            } else {
                model.users = Status::Failed(String::from("No session"));
            }
        },
        Msg::FetchedUsers(res) => {
            match res {
                Ok(users) => {
                    model.users = Status::Ready(
                        users.iter()
                             .map(move |entry| user::Model::from(entry).preview())
                             .collect()
                        );
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::UserPreview(index, msg) => {
            match &mut model.users {
                Status::Ready(users) => {
                    preview::update(
                        msg,
                        &mut users[index],
                        &mut orders.proxy(move |msg| Msg::UserPreview(index.clone(), msg))
                    );
                },
                _ => {}
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.users {
        Status::Ready(users) => {
            div![
                ul![
                    users.iter().enumerate()
                        .map(|(i, u)| li![
                             preview::view(u)
                                .map_msg(move |msg| Msg::UserPreview(i.clone(), msg))
                        ])
                ]
            ]
        },
        Status::Loading => {
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
