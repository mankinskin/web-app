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
    user,
    root::{
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

#[derive(Clone)]
pub struct Model {
    users: Status<Vec<user::Model>>,
    session: Option<UserSession>,
}
impl Model {
    pub fn fetch_all() -> Self {
        Self {
            session: None,
            users: Status::Empty,
        }
    }
}
impl From<Vec<Id<User>>> for Model {
    fn from(users: Vec<Id<User>>) -> Self {
        Self {
            session: None,
            users: Status::Loaded(users
                .iter()
                .map(|id| user::Model::from(*id))
                .collect()),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    FetchUsers,
    FetchedUsers(ResponseDataResult<Vec<Entry<User>>>),
    User(user::Msg),
    SetSession(UserSession),
    EndSession,
}
fn fetch_all_users()
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/api/users")
        .method(Method::Get)
        .fetch_json_data(move |data_result: ResponseDataResult<Vec<Entry<User>>>| {
            Msg::FetchedUsers(data_result)
        })
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::FetchUsers => {
            orders.perform_cmd(fetch_all_users());
            model.users = Status::Loading;
        },
        Msg::FetchedUsers(res) => {
            match res {
                Ok(users) => {
                    model.users = Status::Loaded(
                        users.iter()
                             .map(move |entry| user::Model::from(entry))
                             .collect()
                        );
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::User(_msg) => {

        },
        Msg::SetSession(session) => {
            model.session = Some(session);
        },
        Msg::EndSession => {
            model.session = None;
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.users {
        Status::Loaded(users) => {
            div![
                ul![
                    users.iter()
                        .map(|u| li![user::view(u).map_msg(Msg::User)])
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
    }
}
