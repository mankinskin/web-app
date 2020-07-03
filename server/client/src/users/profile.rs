use rql::{
    *,
};
use crate::{
    config::*,
    users::*,
};
use std::result::Result;

impl Component for Model {
    type Msg = Msg;
}
impl From<Id<User>> for Model {
    fn from(user_id: Id<User>) -> Self {
        Self {
            user_id,
            user: None,
        }
    }
}
impl From<Id<User>> for Msg {
    fn from(id: Id<User>) -> Self {
        Msg::Get(id)
    }
}
impl From<Entry<User>> for Model {
    fn from(entry: Entry<User>) -> Self {
        Self {
            user_id: entry.id().clone(),
            user: Some(entry.data().clone()),
        }
    }
}

#[derive(Clone, Default)]
pub struct Model {
    pub user_id: Id<User>,
    pub user: Option<User>,
    //pub projects: projects::Model,
}
#[derive(Clone)]
pub enum Msg {
    Get(Id<User>),
    User(Result<Option<Entry<User>>, String>),
    //Projects(projects::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Get(id) => {
            orders.perform_cmd(
                api::get_user(id)
                    .map(|res| Msg::User(res.map_err(|e| format!("{:?}", e))))
            );
        },
        Msg::User(res) => {
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
        //Msg::Projects(msg) => {
        //    projects::update(
        //        msg,
        //        &mut model.projects,
        //        &mut orders.proxy(Msg::Projects)
        //    )
        //},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(user) = &model.user {
        div![
            h1!["Profile"],
            p![user.name()],
            p![format!("Followers: {}", user.followers().len())],
            //projects::view(&model.projects)
            //    .map_msg(Msg::Projects),
        ]
    } else {
        div![
            p!["Loading..."],
        ]
    }
}
