use crate::{
    page,
    users::*,
    config::*,
    route,
};
use database::{
    Entry,
};

impl Component for Model {
    type Msg = Msg;
}
impl From<Entry<User>> for Model {
    fn from(entry: Entry<User>) -> Self {
        Self {
            user: entry,
        }
    }
}
#[derive(Clone)]
pub struct Model {
    pub user: Entry<User>,
}
#[derive(Clone)]
pub enum Msg {
    Open,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Open => {
            page::go_to(route::Route::UserProfile(model.user.id().clone()), orders);
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    let user = model.user.data();
    div![
        p!["Preview"],
        a![
            attrs!{
                At::Href => "";
            },
            user.name(),
            simple_ev(Ev::Click, Msg::Open),
        ],
        user.followers().len(),
    ]
}
