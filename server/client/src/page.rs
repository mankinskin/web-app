use crate::*;
use seed::{
    *,
    prelude::*,
};
use plans::{
    user::*,
    project::*,
    task::*,
};
use crate::{
    route::{
        Route,
    },
    root::{
        GMsg,
    },
};
use rql::{
    Id,
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
        Self::home()
    }
}
impl From<Route> for Model {
    fn from(route: Route) -> Self {
        match route {
            Route::Home => Self::home(),
            Route::Login => Self::login(),
            Route::Register => Self::register(),
            Route::Users => Self::users(),
            Route::UserProfile(id) => Self::profile(id),
            Route::Projects=> Self::projects(),
            Route::Project(id) => Self::project(id),
            Route::Task(id) => Self::task(id),
            Route::NotFound => Self::not_found(),
        }
    }
}
impl From<home::Model> for Model {
    fn from(model: home::Model) -> Self {
        Self::Home(model)
    }
}
impl From<login::Model> for Model {
    fn from(model: login::Model) -> Self {
        Self::Login(model)
    }
}
impl From<register::Model> for Model {
    fn from(model: register::Model) -> Self {
        Self::Register(model)
    }
}
impl From<users::Model> for Model {
    fn from(model: users::Model) -> Self {
        Self::Users(model)
    }
}
impl From<users::profile::Model> for Model {
    fn from(model: users::profile::Model) -> Self {
        Self::UserProfile(model)
    }
}
impl From<projects::Model> for Model {
    fn from(model: projects::Model) -> Self {
        Self::Projects(model)
    }
}
impl From<projects::project::Model> for Model {
    fn from(model: projects::project::Model) -> Self {
        Self::Project(model)
    }
}
impl From<tasks::task::Model> for Model {
    fn from(model: tasks::task::Model) -> Self {
        Self::Task(model)
    }
}
impl Model {
    pub fn home() -> Self {
        Self::Home(home::Model::default())
    }
    pub fn users() -> Self {
        Self::Users(users::Model::default())
    }
    pub fn profile(id: Id<User>) -> Self {
        Self::UserProfile(users::user::Model::from(id).profile())
    }
    pub fn projects() -> Self {
        Self::Projects(projects::Model::default())
    }
    pub fn project(id: Id<Project>) -> Self {
        Self::Project(projects::project::Model::from(id))
    }
    pub fn task(id: Id<Task>) -> Self {
        Self::Task(tasks::task::Model::from(id))
    }
    pub fn login() -> Self {
        Self::Login(login::Model::default())
    }
    pub fn register() -> Self {
        Self::Register(register::Model::default())
    }
    pub fn not_found() -> Self {
        Self::NotFound
    }
    pub fn route(&self) -> Route {
        match self {
            Self::UserProfile(profile) => Route::UserProfile(profile.user.user_id),
            Self::Users(_) => Route::Users,
            Self::Project(project) => Route::Project(project.project_id),
            Self::Projects(_) => Route::Projects,
            Self::Task(task) => Route::Task(task.task_id),
            Self::Login(_) => Route::Login,
            Self::Register(_) => Route::Register,
            Self::Home(_) | Self::NotFound => Route::Home,
        }
    }
}
pub fn go_to<M: Into<page::Model> + Clone, Ms: 'static>(model: M, orders: &mut impl Orders<Ms, GMsg>) {
    let page: page::Model = model.into();
    seed::push_route(page.route());
    orders.send_g_msg(GMsg::Root(root::Msg::SetPage(page)));
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
    Fetch,
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
                Msg::Fetch => {
                    users::update(
                        users::Msg::fetch_users(),
                        model,
                        &mut orders.proxy(Msg::Users)
                    );
                }
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
                Msg::Fetch => {
                    users::profile::update(
                        users::profile::Msg::User(users::user::Msg::fetch_user(model.user.user_id)),
                        model,
                        &mut orders.proxy(Msg::UserProfile)
                    );
                }
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
                Msg::Fetch => {
                    projects::update(
                        projects::Msg::fetch_all(),
                        model,
                        &mut orders.proxy(Msg::Projects)
                    );
                }
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
                Msg::Fetch => {
                    projects::project::update(
                        projects::project::Msg::fetch_project(model.project_id),
                        model,
                        &mut orders.proxy(Msg::Project)
                    );
                }
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
                Msg::Fetch => {
                    tasks::task::update(
                        tasks::task::Msg::fetch_task(model.task_id),
                        model,
                        &mut orders.proxy(Msg::Task)
                    );
                }
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
