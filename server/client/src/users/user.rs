use rql::{
    *,
};
use crate::{
    root,
    config::*,
    users::*,
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Id<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            user_id: self.clone(),
            user: None,
        }
    }
    fn send_msg(self, orders: &mut impl Orders<Msg, root::GMsg>) {
        orders.send_msg(Msg::Get(self));
    }
}
impl Config<Model> for Entry<User> {
    fn into_model(self, _orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        let id = self.id().clone();
        let data = self.data().clone();
        Model {
            user_id: id.clone(),
            user: Some(data),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}

#[derive(Clone, Default)]
pub struct Model {
    pub user_id: Id<User>,
    pub user: Option<User>,
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<User>),
    GotUser(Result<Option<Entry<User>>, String>),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_user(id)
                    .map(|res| Msg::GotUser(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::GotUser(res) => {
            match res {
                Ok(r) => {
                    if let Some(entry) = r {
                        model.user_id = entry.id().clone();
                        model.user = Some(entry.data().clone());
                    }
                },
                Err(e) => { seed::log(e); },
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(user) = &model.user {
        div![
            h1!["Profile"],
            p![user.name()],
            p![format!("Followers: {}", user.followers().len())],
        ]
    } else {
        div![
            p!["Loading..."],
        ]
    }
}
