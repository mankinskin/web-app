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

#[derive(Clone)]
pub struct Model {
    users: Status<Vec<user::Model>>,
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
}
fn fetch_all_users()
    -> impl Future<Output = Result<Msg, Msg>>
{
    let mut request = Request::new("http://localhost:8000/api/users");
    if let Some(session) = root::get_session() {
        request = request.header("authorization", &format!("{}", session.token));
    }
    request.method(Method::Get)
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
