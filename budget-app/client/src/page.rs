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

#[derive(Debug, Clone)]
pub enum Page {
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
impl Default for Page {
	fn default() -> Self {
		Self::Home(home::Model::default())
	}
}
impl Init<Route> for Page {
	fn init(route: Route, orders: &mut impl Orders<Msg>) -> Self {
		match route {
			Route::Root => Page::Home(Default::default()),
			Route::Auth(route) => {
				match route {
					auth::Route::Login => Self::Login(Default::default()),
					auth::Route::Register => Self::Register(Default::default()),
				}
			}
			Route::User(route) => {
				match route {
					user::Route::Users => {
						Self::UserList(Init::init(
							list::Msg::GetAll,
							&mut orders.proxy(Msg::UserList),
						))
					}
					user::Route::User(id) => {
						Self::UserProfile(Init::init(id, &mut orders.proxy(Msg::UserProfile)))
					}
				}
			}
			Route::Project(route) => {
				match route {
					project::Route::Projects => {
						Self::ProjectList(Init::init(
							list::Msg::GetAll,
							&mut orders.proxy(Msg::ProjectList),
						))
					}
					project::Route::Project(id) => {
						Self::ProjectProfile(Init::init(id, &mut orders.proxy(Msg::ProjectProfile)))
					}
				}
			}
			Route::Task(route) => {
				match route {
					task::Route::Task(id) => {
						Self::TaskProfile(Init::init(id, &mut orders.proxy(Msg::TaskProfile)))
					}
					_ => Self::Home(Default::default()),
				}
			}
			Route::NotFound => Self::NotFound,
		}
	}
}
impl From<Page> for Route {
	fn from(page: Page) -> Self {
		match page {
			Page::Home(_) | Page::NotFound => Route::Root,
			Page::Login(_) => Route::Auth(auth::Route::Login),
			Page::Register(_) => Route::Auth(auth::Route::Register),
			Page::UserList(_) => Route::User(user::Route::Users),
			Page::UserProfile(profile) => Route::User(profile.entry.route()),
			Page::ProjectProfile(profile) => Route::Project(profile.entry.route()),
			Page::ProjectList(_) => Route::Project(project::Route::Projects),
			Page::TaskProfile(profile) => Route::Task(profile.entry.route()),
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
impl Component for Page {
	type Msg = Msg;
	fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
		match msg {
			Msg::GoTo(route) => {
				*self = Init::init(route, orders);
			}
			Msg::Home(msg) => {
				match self {
					Self::Home(home) => {
						home.update(msg, &mut orders.proxy(Msg::Home));
					}
					_ => {}
				}
			}
			Msg::Login(msg) => {
				match self {
					Self::Login(login) => {
						login.update(msg, &mut orders.proxy(Msg::Login));
					}
					_ => {}
				}
			}
			Msg::Register(msg) => {
				match self {
					Self::Register(register) => {
						register.update(msg, &mut orders.proxy(Msg::Register));
					}
					_ => {}
				}
			}
			Msg::UserList(msg) => {
				match self {
					Self::UserList(list) => {
						list.update(msg, &mut orders.proxy(Msg::UserList));
					}
					_ => {}
				}
			}
			Msg::UserProfile(msg) => {
				match self {
					Self::UserProfile(profile) => {
						profile.update(msg, &mut orders.proxy(Msg::UserProfile));
					}
					_ => {}
				}
			}
			Msg::ProjectList(msg) => {
				match self {
					Self::ProjectList(list) => {
						list.update(msg, &mut orders.proxy(Msg::ProjectList));
					}
					_ => {}
				}
			}
			Msg::ProjectProfile(msg) => {
				match self {
					Self::ProjectProfile(profile) => {
						profile.update(msg, &mut orders.proxy(Msg::ProjectProfile));
					}
					_ => {}
				}
			}
			Msg::TaskProfile(msg) => {
				match self {
					Self::TaskProfile(profile) => {
						profile.update(msg, &mut orders.proxy(Msg::TaskProfile));
					}
					_ => {}
				}
			}
		}
	}
}
impl Viewable for Page {
	fn view(&self) -> Node<Msg> {
		match self {
			Self::NotFound => div!["Not Found"],
			Self::Home(model) => model.view().map_msg(Msg::Home),
			Self::Login(model) => model.view().map_msg(Msg::Login),
			Self::Register(model) => model.view().map_msg(Msg::Register),
			Self::UserList(model) => model.view().map_msg(Msg::UserList),
			Self::UserProfile(model) => model.view().map_msg(Msg::UserProfile),
			Self::ProjectList(model) => model.view().map_msg(Msg::ProjectList),
			Self::ProjectProfile(model) => model.view().map_msg(Msg::ProjectProfile),
			Self::TaskProfile(model) => model.view().map_msg(Msg::TaskProfile),
		}
	}
}
