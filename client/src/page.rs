use crate::home;
use crate::*;
use app_model::{
    auth::{
        self,
        login,
        register,
    },
    project,
    task,
    user,
    user::User,
};
use components::{
    list,
    Component,
    Init,
    Viewable,
};
use database_table::Routable;
use seed::{
    prelude::*,
    *,
};

#[derive(Clone)]
pub enum Model {
    NotFound,
    Home(home::Model),
    Login(login::Login),
    Register(register::Register),
    UserProfile(user::profile::Model),
    UserList(list::List<User>),
    ProjectList(project::list::Model),
    ProjectProfile(project::profile::Model),
    TaskProfile(task::profile::Model),
}
impl Default for Model {
    fn default() -> Self {
        Self::Home(home::Model::default())
    }
}
impl Init<Route> for Model {
    fn init(route: Route, orders: &mut impl Orders<Msg>) -> Model {
        match route {
            Route::Root => Model::Home(Default::default()),
            Route::Auth(route) => {
                match route {
                    auth::Route::Login => Model::Login(Default::default()),
                    auth::Route::Register => Model::Register(Default::default()),
                }
            }
            Route::User(route) => {
                match route {
                    user::Route::Users => {
                        Model::UserList(Init::init(
                            list::Msg::GetAll,
                            &mut orders.proxy(Msg::UserList),
                        ))
                    }
                    user::Route::User(id) => {
                        Model::UserProfile(Init::init(id, &mut orders.proxy(Msg::UserProfile)))
                    }
                }
            }
            Route::Project(route) => {
                match route {
                    project::Route::Projects => {
                        Model::ProjectList(Init::init(
                            list::Msg::GetAll,
                            &mut orders.proxy(Msg::ProjectList),
                        ))
                    }
                    project::Route::Project(id) => {
                        Model::ProjectProfile(Init::init(id, &mut orders.proxy(Msg::ProjectProfile)))
                    }
                }
            }
            Route::Task(route) => {
                match route {
                    task::Route::Task(id) => {
                        Model::TaskProfile(Init::init(id, &mut orders.proxy(Msg::TaskProfile)))
                    }
                    _ => Model::Home(Default::default()),
                }
            }
        }
    }
}
impl From<page::Model> for Route {
    fn from(components: page::Model) -> Self {
        match components {
            Model::Home(_) | page::Model::NotFound => Route::Root,
            Model::Login(_) => Route::Auth(auth::Route::Login),
            Model::Register(_) => Route::Auth(auth::Route::Register),
            Model::UserList(_) => Route::User(user::Route::Users),
            Model::UserProfile(profile) => Route::User(profile.entry.route()),
            Model::ProjectProfile(profile) => Route::Project(profile.entry.route()),
            Model::ProjectList(_) => Route::Project(project::Route::Projects),
            Model::TaskProfile(profile) => Route::Task(profile.entry.route()),
        }
    }
}
#[derive(Debug, Clone)]
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
                *self = Init::init(route, orders);
            }
            Msg::Home(msg) => {
                match self {
                    Model::Home(home) => {
                        home.update(msg, &mut orders.proxy(Msg::Home));
                    }
                    _ => {}
                }
            }
            Msg::Login(msg) => {
                match self {
                    Model::Login(login) => {
                        login.update(msg, &mut orders.proxy(Msg::Login));
                    }
                    _ => {}
                }
            }
            Msg::Register(msg) => {
                match self {
                    Model::Register(register) => {
                        register.update(msg, &mut orders.proxy(Msg::Register));
                    }
                    _ => {}
                }
            }
            Msg::UserList(msg) => {
                match self {
                    Model::UserList(list) => {
                        list.update(msg, &mut orders.proxy(Msg::UserList));
                    }
                    _ => {}
                }
            }
            Msg::UserProfile(msg) => {
                match self {
                    Model::UserProfile(profile) => {
                        profile.update(msg, &mut orders.proxy(Msg::UserProfile));
                    }
                    _ => {}
                }
            }
            Msg::ProjectList(msg) => {
                match self {
                    Model::ProjectList(list) => {
                        list.update(msg, &mut orders.proxy(Msg::ProjectList));
                    }
                    _ => {}
                }
            }
            Msg::ProjectProfile(msg) => {
                match self {
                    Model::ProjectProfile(profile) => {
                        profile.update(msg, &mut orders.proxy(Msg::ProjectProfile));
                    }
                    _ => {}
                }
            }
            Msg::TaskProfile(msg) => {
                match self {
                    Model::TaskProfile(profile) => {
                        profile.update(msg, &mut orders.proxy(Msg::TaskProfile));
                    }
                    _ => {}
                }
            }
        }
    }
}
impl Viewable for Model {
    fn view(&self) -> Node<Msg> {
        match self {
            Model::NotFound => div!["Not Found"],
            Model::Home(model) => model.view().map_msg(Msg::Home),
            Model::Login(model) => model.view().map_msg(Msg::Login),
            Model::Register(model) => model.view().map_msg(Msg::Register),
            Model::UserList(model) => model.view().map_msg(Msg::UserList),
            Model::UserProfile(model) => model.view().map_msg(Msg::UserProfile),
            Model::ProjectList(model) => model.view().map_msg(Msg::ProjectList),
            Model::ProjectProfile(model) => model.view().map_msg(Msg::ProjectProfile),
            Model::TaskProfile(model) => model.view().map_msg(Msg::TaskProfile),
        }
    }
}
