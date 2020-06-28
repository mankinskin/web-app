use crate::*;
use seed::{
    *,
    prelude::*,
};
use crate::{
    route::{
        Route,
    },
    root::{
        GMsg,
    },
};

#[derive(Clone)]
pub enum Model {
    Login(login::Model),
    Register(register::Model),
    Home(home::Model),
    UserProfile(users::profile::Model),
    Users(users::Model),
    Projects(projects::Model),
    Project(projects::project::Model),
    Task(tasks::task::Model),
    NotFound,
}
impl Default for Model {
    fn default() -> Self {
        Self::Home(home::Model::default())
    }
}
pub fn init(config: Config, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    match config {
        Config::Home(c) => Model::Home(home::init(c, &mut orders.proxy(Msg::Home))),
        Config::Users(c) => Model::Users(users::init(c, &mut orders.proxy(Msg::Users))),
        Config::UserProfile(c) => Model::UserProfile(users::profile::init(c, &mut orders.proxy(Msg::UserProfile))),
        Config::Projects(c) => Model::Projects(projects::init(c, &mut orders.proxy(Msg::Projects))),
        Config::Project(c) => Model::Project(projects::project::init(c, &mut orders.proxy(Msg::Project))),
        Config::Task(c) => Model::Task(tasks::task::init(c, &mut orders.proxy(Msg::Task))),
        Config::Login(c) => Model::Login(login::init(c, &mut orders.proxy(Msg::Login))),
        Config::Register(c) => Model::Register(register::init(c, &mut orders.proxy(Msg::Register))),
    }
}
pub fn go_to<C: Into<page::Config> + Clone, Ms: 'static>(config: C, orders: &mut impl Orders<Ms, GMsg>) {
    let config: page::Config = config.into();
    seed::push_route(Route::from(config.clone()));
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(config)));
}

#[derive(Clone)]
pub enum Config {
    Home(home::Config),
    Login(login::Config),
    Register(register::Config),
    Users(users::Config),
    UserProfile(users::profile::Config),
    Projects(projects::Config),
    Project(projects::project::Config),
    Task(tasks::task::Config),
}
impl From<home::Config> for Config {
    fn from(config: home::Config) -> Self {
        Self::Home(config)
    }
}
impl From<login::Config> for Config {
    fn from(config: login::Config) -> Self {
        Self::Login(config)
    }
}
impl From<register::Config> for Config {
    fn from(config: register::Config) -> Self {
        Self::Register(config)
    }
}
impl From<users::Config> for Config {
    fn from(config: users::Config) -> Self {
        Self::Users(config)
    }
}
impl From<users::profile::Config> for Config {
    fn from(config: users::profile::Config) -> Self {
        Self::UserProfile(config)
    }
}
impl From<projects::Config> for Config {
    fn from(config: projects::Config) -> Self {
        Self::Projects(config)
    }
}
impl From<projects::project::Config> for Config {
    fn from(config: projects::project::Config) -> Self {
        Self::Project(config)
    }
}
impl From<tasks::task::Config> for Config {
    fn from(config: tasks::task::Config) -> Self {
        Self::Task(config)
    }
}
impl From<seed::Url> for Config {
    fn from(url: seed::Url) -> Self {
        Self::from(Route::from(url))
    }
}
impl From<Route> for Config {
    fn from(route: Route) -> Self {
        match route {
            Route::Login => Self::Login(login::Config::default()),
            Route::Register => Self::Register(register::Config::default()),
            Route::Users => Self::Users(users::Config::All),
            Route::UserProfile(id) => Self::UserProfile(users::profile::Config::UserId(id)),
            Route::Projects => Self::Projects(projects::Config::All),
            Route::Project(id) => Self::Project(projects::project::Config::ProjectId(id)),
            Route::Task(id) => Self::Task(tasks::task::Config::TaskId(id)),
            Route::Home | Route::NotFound => Self::Home(home::Config::default()),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    Login(login::Msg),
    Register(register::Msg),
    Users(users::Msg),
    UserProfile(users::profile::Msg),
    Projects(projects::Msg),
    Project(projects::project::Msg),
    Task(tasks::task::Msg),
}
impl From<home::Msg> for Msg {
    fn from(msg: home::Msg) -> Self {
        Self::Home(msg)
    }
}
impl From<users::profile::Msg> for Msg {
    fn from(msg: users::profile::Msg) -> Self {
        Self::UserProfile(msg)
    }
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
impl From<register::Msg> for Msg {
    fn from(msg: register::Msg) -> Self {
        Self::Register(msg)
    }
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
        Model::Users(model) => {
            match msg {
                Msg::Users(msg) => {
                    users::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::Users)
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
        Model::Projects(model) => {
            match msg {
                Msg::Projects(msg) => {
                    projects::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::Projects)
                    );
                },
                _ => {}
            }
        },
        Model::Project(model) => {
            match msg {
                Msg::Project(msg) => {
                    projects::project::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::Project)
                    );
                },
                _ => {}
            }
        },
        Model::Task(model) => {
            match msg {
                Msg::Task(msg) => {
                    tasks::task::update(
                        msg,
                        model,
                        &mut orders.proxy(Msg::Task)
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
        Model::Home(model) =>
            home::view(&model)
                .map_msg(Msg::Home),
        Model::Login(model) =>
            login::view(&model)
                .map_msg(Msg::Login),
        Model::Register(model) =>
            register::view(&model)
                .map_msg(Msg::Register),
        Model::Users(model) =>
            users::view(&model)
                .map_msg(Msg::Users),
        Model::UserProfile(model) =>
            users::profile::view(&model)
                .map_msg(Msg::UserProfile),
        Model::Projects(model) =>
            projects::view(&model)
                .map_msg(Msg::Projects),
        Model::Project(model) =>
            projects::project::view(&model)
                .map_msg(Msg::Project),
        Model::Task(model) =>
            tasks::task::view(&model)
                .map_msg(Msg::Task),
        Model::NotFound =>
            div!["Not Found"]
    }
}
