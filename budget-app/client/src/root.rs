use crate::{
	navbar,
};
use app_model::{
	auth::{
		self,
	},
	user,
	project,
	task,
};
use components::{
	Component,
	Init,
	Viewable,
};
use database_table::Routable;
use enum_paths::{
	AsPath,
	ParsePath,
};
use seed::{
	prelude::*,
	*,
};

#[wasm_bindgen(start)]
pub fn render() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	App::start(
		"app",
		|url, orders| Model::init(url, orders),
		|msg, model, orders| model.update(msg, orders),
		Viewable::view,
	);
}
#[derive(Debug)]
pub struct Model {
	navbar: navbar::Navbar,
}
#[derive(Debug, Clone, AsPath)]
pub enum Route {
	#[as_path = ""]
	Auth(auth::Route),
	User(user::Route),
	Project(project::Route),
	Task(task::Route),
	NotFound,
	#[as_path = ""]
	Root,
}
impl Default for Route {
	fn default() -> Self {
		Self::Root
	}
}
impl components::Route for Route {}

impl Init<Url> for Model {
	fn init(url: Url, orders: &mut impl Orders<Self::Msg>) -> Model {
		let route = Route::parse_path(&url.to_string()).unwrap_or(Route::NotFound);
		Model::init(route, orders)
	}
}
impl Init<Route> for Model {
	fn init(route: Route, orders: &mut impl Orders<Self::Msg>) -> Model {
		orders.subscribe(|msg: Msg| msg);
		Model {
			navbar: Init::init(route, &mut orders.proxy(Msg::Navbar)),
		}
	}
}
#[derive(Debug, Clone)]
pub enum Msg {
	Navbar(navbar::Msg),
}
fn refresh_session() {
	if let None = auth::session::get() {
		if let Some(session) = auth::session::load() {
			auth::session::set(session);
		}
	}
}
pub fn go_to<R: Routable, Ms: 'static>(_r: R, _orders: &mut impl Orders<Ms>) {
	//orders.notify(subs::UrlRequested::new(r.route().into()));
}
impl Component for Model {
	type Msg = Msg;
	fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
		refresh_session();
		seed::log!(msg);
		match msg {
			Msg::Navbar(msg) => {
				self.navbar.update(msg, &mut orders.proxy(Msg::Navbar));
			}
		}
	}
}
impl Viewable for Model {
	fn view(&self) -> Node<Self::Msg> {
		div![
			self.navbar.view().map_msg(Msg::Navbar),
		]
	}
}
