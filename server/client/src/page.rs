use crate::*;
use seed::{
    *,
    prelude::*,
};
use plans::{
    user::User,
};
use crate::{
    config::{
        Component,
        Config,
        View,
    },
    route::{
        Route,
        Routable,
    },
    root::{
        GMsg,
    },
};


#[derive(Clone)]
pub enum Model {
    NotFound,
    Home(home::Model),
    Login(login::Model),
    Register(register::Model),
    UserProfile(users::profile::Model),
    UserList(list::Model<User>),
    ProjectList(projects::list::Model),
    ProjectProfile(projects::profile::Model),
    TaskProfile(tasks::profile::Model),
}
impl Default for Model {
    fn default() -> Self {
        Self::Home(home::Model::default())
    }
}
impl Config<Model> for Route {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        match self {
            Route::NotFound => Model::Home(Default::default()),
            Route::Home => Model::Home(Default::default()),
            Route::Login => Model::Login(Default::default()),
            Route::Register => Model::Register(Default::default()),
            Route::Users => Model::UserList(Config::init(list::Msg::GetAll, &mut orders.proxy(Msg::UserList))),
            Route::User(id) => Model::UserProfile(Config::init(id, &mut orders.proxy(Msg::UserProfile))),
            Route::Projects => Model::ProjectList(Config::init(projects::list::Msg::GetAll, &mut orders.proxy(Msg::ProjectList))),
            Route::Project(id) => Model::ProjectProfile(Config::init(id, &mut orders.proxy(Msg::ProjectProfile))),
            Route::Task(id) => Model::TaskProfile(Config::init(id, &mut orders.proxy(Msg::TaskProfile))),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
pub fn go_to<R: Routable, Ms: 'static>(r: R, orders: &mut impl Orders<Ms, GMsg>) {
    let route = r.route();
    seed::push_route(route.clone());
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(route)));
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    Login(login::Msg),
    Register(register::Msg),
    UserList(list::Msg<User>),
    UserProfile(users::profile::Msg),
    ProjectList(projects::list::Msg),
    ProjectProfile(projects::profile::Msg),
    TaskProfile(tasks::profile::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match self {
            Model::Home(home) => {
                match msg {
                    Msg::Home(msg) => {
                        home.update(
                            msg,
                            &mut orders.proxy(Msg::Home)
                        );
                    },
                    _ => {}
                }
            },
            Model::Login(login) => {
                match msg {
                    Msg::Login(msg) => {
                        login.update(
                            msg,
                            &mut orders.proxy(Msg::Login)
                        );
                    },
                    _ => {}
                }
            },
            Model::Register(register) => {
                match msg {
                    Msg::Register(msg) => {
                        register.update(
                            msg,
                            &mut orders.proxy(Msg::Register)
                        );
                    },
                    _ => {}
                }
            },
            Model::UserList(list) => {
                match msg {
                    Msg::UserList(msg) => {
                        list.update(
                            msg,
                            &mut orders.proxy(Msg::UserList)
                        );
                    },
                    _ => {}
                }
            },
            Model::UserProfile(profile) => {
                match msg {
                    Msg::UserProfile(msg) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::UserProfile)
                        );
                    },
                    _ => {}
                }
            },
            Model::ProjectList(list) => {
                match msg {
                    Msg::ProjectList(msg) => {
                        list.update(
                            msg,
                            &mut orders.proxy(Msg::ProjectList)
                        );
                    },
                    _ => {}
                }
            },
            Model::ProjectProfile(profile) => {
                match msg {
                    Msg::ProjectProfile(msg) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::ProjectProfile)
                        );
                    },
                    _ => {}
                }
            },
            Model::TaskProfile(profile) => {
                match msg {
                    Msg::TaskProfile(msg) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::TaskProfile)
                        );
                    },
                    _ => {}
                }
            },
            _ => {},
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        match self {
            Model::NotFound =>
                div!["Not Found"],
            Model::Home(model) =>
                model.view().map_msg(Msg::Home),
            Model::Login(model) =>
                model.view().map_msg(Msg::Login),
            Model::Register(model) =>
                model.view().map_msg(Msg::Register),
            Model::UserList(model) =>
                model.view().map_msg(Msg::UserList),
            Model::UserProfile(model) =>
                model.view().map_msg(Msg::UserProfile),
            Model::ProjectList(model) =>
                model.view().map_msg(Msg::ProjectList),
            Model::ProjectProfile(model) =>
                model.view().map_msg(Msg::ProjectProfile),
            Model::TaskProfile(model) =>
                model.view().map_msg(Msg::TaskProfile),
        }
    }
}
