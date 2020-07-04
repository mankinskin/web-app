use crate::*;
use seed::{
    *,
    prelude::*,
};
use crate::{
    config::*,
    route::{
        Route,
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
    UserList(users::list::Model),
    ProjectList(projects::list::Model),
    ProjectProfile(projects::profile::Model),
    TaskProfile(tasks::profile::Model),
}
impl Default for Model {
    fn default() -> Self {
        Self::Home(home::Model::default())
    }
}
impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Route {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        match self {
            Route::NotFound => Model::Home(Default::default()),
            Route::Home => Model::Home(Default::default()),
            Route::Login => Model::Login(Default::default()),
            Route::Register => Model::Register(Default::default()),
            Route::Users => Model::UserList(Config::init(users::list::Msg::GetAll, &mut orders.proxy(Msg::UserList))),
            Route::User(id) => Model::UserProfile(Config::init(id, &mut orders.proxy(Msg::UserProfile))),
            Route::Projects => Model::ProjectList(Config::init(projects::list::Msg::GetAll, &mut orders.proxy(Msg::ProjectList))),
            Route::Project(id) => Model::ProjectProfile(Config::init(id, &mut orders.proxy(Msg::ProjectProfile))),
            Route::Task(id) => Model::TaskProfile(Config::init(id, &mut orders.proxy(Msg::TaskProfile))),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
pub fn go_to<Ms: 'static>(route: Route, orders: &mut impl Orders<Ms, GMsg>) {
    seed::push_route(route.clone());
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(route)));
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    Login(login::Msg),
    Register(register::Msg),
    UserList(users::list::Msg),
    UserProfile(users::profile::Msg),
    ProjectList(projects::list::Msg),
    ProjectProfile(projects::profile::Msg),
    TaskProfile(tasks::profile::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match model {
        Model::Home(home) => {
            match msg {
                Msg::Home(msg) => {
                    home::update(
                        msg,
                        home,
                        &mut orders.proxy(Msg::Home)
                    );
                },
                _ => {}
            }
        },
        Model::Login(login) => {
            match msg {
                Msg::Login(msg) => {
                    login::update(
                        msg,
                        login,
                        &mut orders.proxy(Msg::Login)
                    );
                },
                _ => {}
            }
        },
        Model::Register(register) => {
            match msg {
                Msg::Register(msg) => {
                    register::update(
                        msg,
                        register,
                        &mut orders.proxy(Msg::Register)
                    );
                },
                _ => {}
            }
        },
        Model::UserList(model) => {
            match msg {
                Msg::UserList(msg) => {
                    users::list::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::UserList)
                    );
                },
                _ => {}
            }
        },
        Model::UserProfile(model) => {
            match msg {
                Msg::UserProfile(msg) => {
                    users::profile::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::UserProfile)
                    );
                },
                _ => {}
            }
        },
        Model::ProjectList(model) => {
            match msg {
                Msg::ProjectList(msg) => {
                    projects::list::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::ProjectList)
                    );
                },
                _ => {}
            }
        },
        Model::ProjectProfile(model) => {
            match msg {
                Msg::ProjectProfile(msg) => {
                    projects::profile::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::ProjectProfile)
                    );
                },
                _ => {}
            }
        },
        Model::TaskProfile(model) => {
            match msg {
                Msg::TaskProfile(msg) => {
                    tasks::profile::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::TaskProfile)
                    );
                },
                _ => {}
            }
        },
        _ => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::NotFound =>
            div!["Not Found"],
        Model::Home(model) =>
            home::view(&model)
                .map_msg(Msg::Home),
        Model::Login(model) =>
            login::view(&model)
                .map_msg(Msg::Login),
        Model::Register(model) =>
            register::view(&model)
                .map_msg(Msg::Register),
        Model::UserList(model) =>
            users::list::view(&model)
                .map_msg(Msg::UserList),
        Model::UserProfile(model) =>
            users::profile::view(&model)
                .map_msg(Msg::UserProfile),
        Model::ProjectList(model) =>
            projects::list::view(&model)
                .map_msg(Msg::ProjectList),
        Model::ProjectProfile(model) =>
            projects::profile::view(&model)
                .map_msg(Msg::ProjectProfile),
        Model::TaskProfile(model) =>
            tasks::profile::view(&model)
                .map_msg(Msg::TaskProfile),
    }
}
