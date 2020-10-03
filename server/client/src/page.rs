use crate::*;
use seed::{
    *,
    prelude::*,
};
use plans::{
    user::User,
};
use crate::{
    components::{
        Component,
        Config,
        View,
    },
};
use api::{
    routes::{
        Route,
        Routable,
    },
};


#[derive(Clone)]
pub enum Model {
    NotFound,
    Home(home::Model),
    Login(login::Model),
    Register(register::Model),
    UserProfile(user::profile::Model),
    UserList(list::Model<User>),
    ProjectList(project::list::Model),
    ProjectProfile(project::profile::Model),
    TaskProfile(task::profile::Model),
}
impl Default for Model {
    fn default() -> Self {
        Self::Home(home::Model::default())
    }
}
impl Config<Model> for Route {
    fn init(self, orders: &mut impl Orders<Msg>) -> Model {
        match self {
            Route::NotFound => Model::Home(Default::default()),
            Route::Home => Model::Home(Default::default()),
            Route::Login => Model::Login(Default::default()),
            Route::Register => Model::Register(Default::default()),
            Route::Users => Model::UserList(Config::init(list::Msg::GetAll, &mut orders.proxy(Msg::UserList))),
            Route::User(id) => Model::UserProfile(Config::init(id, &mut orders.proxy(Msg::UserProfile))),
            Route::Projects => Model::ProjectList(Config::init(list::Msg::GetAll, &mut orders.proxy(Msg::ProjectList))),
            Route::Project(id) => Model::ProjectProfile(Config::init(id, &mut orders.proxy(Msg::ProjectProfile))),
            Route::Task(id) => Model::TaskProfile(Config::init(id, &mut orders.proxy(Msg::TaskProfile))),
        }
    }
}
impl From<page::Model> for Route {
    fn from(config: page::Model) -> Self {
        match config {
            Model::Home(_) | page::Model::NotFound => Route::Home,
            Model::Login(_) => Route::Login,
            Model::Register(_) => Route::Register,
            Model::UserList(_) => Route::Users,
            Model::UserProfile(profile) => profile.entry.route(),
            Model::ProjectProfile(profile) => profile.entry.route(),
            Model::ProjectList(_) => Route::Projects,
            Model::TaskProfile(profile) => profile.entry.route(),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    Home(home::Msg),
    Login(login::Msg),
    Register(register::Msg),
    UserList(list::Msg<User>),
    UserProfile(user::profile::Msg),
    ProjectList(project::list::Msg),
    ProjectProfile(project::profile::Msg),
    TaskProfile(task::profile::Msg),
    GoTo(Route),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::GoTo(route) => {
                *self = Config::init(route, orders);
            },
            Msg::Home(msg) => {
                match self {
                    Model::Home(home) => {
                        home.update(
                            msg,
                            &mut orders.proxy(Msg::Home)
                        );
                    },
                    _ => {}
                }
            },
            Msg::Login(msg) => {
                match self {
                    Model::Login(login) => {
                        login.update(
                            msg,
                            &mut orders.proxy(Msg::Login)
                        );
                    },
                    _ => {}
                }
            },
            Msg::Register(msg) => {
                match self {
                    Model::Register(register) => {
                        register.update(
                            msg,
                            &mut orders.proxy(Msg::Register)
                        );
                    },
                    _ => {}
                }
            },
            Msg::UserList(msg) => {
                match self {
                    Model::UserList(list) => {
                        list.update(
                            msg,
                            &mut orders.proxy(Msg::UserList)
                        );
                    },
                    _ => {}
                }
            },
            Msg::UserProfile(msg) => {
                match self {
                    Model::UserProfile(profile) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::UserProfile)
                        );
                    },
                    _ => {}
                }
            },
            Msg::ProjectList(msg) => {
                match self {
                    Model::ProjectList(list) => {
                        list.update(
                            msg,
                            &mut orders.proxy(Msg::ProjectList)
                        );
                    },
                    _ => {}
                }
            },
            Msg::ProjectProfile(msg) => {
                match self {
                    Model::ProjectProfile(profile) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::ProjectProfile)
                        );
                    },
                    _ => {}
                }
            },
            Msg::TaskProfile(msg) => {
                match self {
                    Model::TaskProfile(profile) => {
                        profile.update(
                            msg,
                            &mut orders.proxy(Msg::TaskProfile)
                        );
                    },
                    _ => {}
                }
            },
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
