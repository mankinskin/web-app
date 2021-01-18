#![feature(clamp)]
pub mod chart;
pub mod page;
pub mod navbar;
pub mod websocket;
pub mod subscriptions;
pub mod webapi;

use wasm_bindgen_test::wasm_bindgen_test;
use components::{
	Component,
	Init,
	Viewable,
};
use navbar::Navbar;
use webapi::WebApi;
use seed::{
	prelude::*,
	*,
};
#[allow(unused)]
use tracing::{
	debug,
	error,
	info,
	trace,
};

fn init_tracing() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	debug!("Tracing initialized.");
	debug!("Debug logs enabled.");
	info!("Info logs enabled.");
	trace!("Trace logs enabled.");
	error!("Error logs enabled.");
}
#[wasm_bindgen(start)]
pub fn render() {
	init_tracing();
	debug!("Starting App");
	App::start(
		"app",
		Root::init,
		|msg, model, orders| model.update(msg, orders),
		Viewable::view,
	);
}
struct Root {
	webapi: WebApi,
	navbar: Navbar,
}
#[derive(Clone, Debug)]
enum Msg {
	Navbar(navbar::Msg),
	WebApi(webapi::Msg),
}
impl Init<Url> for Root {
	fn init(url: Url, orders: &mut impl Orders<Msg>) -> Self {
		Self {
			navbar: Navbar::init(url, &mut orders.proxy(Msg::Navbar)),
			webapi: WebApi::init((), &mut orders.proxy(Msg::WebApi)),
		}
	}
}
impl Component for Root {
	type Msg = Msg;
	fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
		//debug!("Root Update");
		match msg {
			Msg::Navbar(msg) => self.navbar.update(msg, &mut orders.proxy(Msg::Navbar)),
			Msg::WebApi(msg) => self.webapi.update(msg, &mut orders.proxy(Msg::WebApi)),
		}
	}
}
impl Viewable for Root {
	fn view(&self) -> Node<Msg> {
		div![
			self.navbar.view().map_msg(Msg::Navbar)
		]
	}
}
#[allow(unused)]
fn mutation_observer() {
	if let Some(node) = web_sys::window()
		.and_then(|w| w.document())
		.and_then(|document| document.get_element_by_id("graph-svg"))
	{
		debug!("found node");
		let closure = wasm_bindgen::closure::Closure::new(|record: web_sys::MutationRecord| {
			debug!("Mutation {:?}", record);
		});
		let function = js_sys::Function::from(closure.into_js_value());
		let observer = web_sys::MutationObserver::new(&function).unwrap();
		observer.observe(&node).unwrap();
	}
}
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
async fn basic_test() {
    let response = fetch("/basic").await;
    debug!("{:#?}", response);
}
