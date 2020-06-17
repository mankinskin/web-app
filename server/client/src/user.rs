use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
};
use rql::{
    *,
};
use database::{
    Entry,
};
use futures::{
    Future,
};
use std::result::{
    Result,
};
use crate::{
    root::{
        self,
        GMsg,
    },
};

#[derive(Clone, Default)]
pub struct Model {
    user_id: Id<User>,
    user: Option<User>,
}
impl From<Id<User>> for Model {
    fn from(user_id: Id<User>) -> Self {
        Self {
            user_id,
            user: None,
        }
    }
}
impl From<&Entry<User>> for Model {
    fn from(entry: &Entry<User>) -> Self {
        Self {
            user_id: *entry.id(),
            user: Some(entry.data().clone()),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    FetchUser,
    FetchedUser(ResponseDataResult<User>),
}
fn fetch_user(id: Id<User>)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new(format!("http://localhost:8000/api/users/{}", id))
        .method(Method::Get)
        .fetch_json_data(move |data_result: ResponseDataResult<User>| {
            Msg::FetchedUser(data_result)
        })
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::FetchUser => {
            orders.perform_cmd(fetch_user(model.user_id));
        },
        Msg::FetchedUser(res) => {
            match res {
                Ok(user) => {
                    model.user = Some(user);
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(user) = &model.user {
        div![
            p![format!("{}", model.user_id)],
            p![user.name()],
        ]
    } else {
        div![
            p![format!("User {} not found", model.user_id)]
        ]
    }
}
