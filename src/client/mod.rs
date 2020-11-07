pub mod chart;
pub mod page;
pub mod navbar;
pub mod websocket;
pub mod subscriptions;

use components::{
    Component,
    Init,
    Viewable,
};
use navbar::Navbar;
use seed::{
    prelude::*,
    *,
};
use tracing::debug;
use crate::websocket::{
    WebSocket,
};

fn init_tracing() {
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
}
struct Root {
    navbar: Navbar,
    websocket: WebSocket,
}
#[derive(Clone, Debug)]
enum Msg {
    Navbar(navbar::Msg),
    Websocket(websocket::Msg),
}
impl Init<Url> for Root {
    fn init(url: Url, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            navbar: Navbar::init(url, &mut orders.proxy(Msg::Navbar)),
            websocket: WebSocket::init((), &mut orders.proxy(Msg::Websocket)),
        }
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        //debug!("Root Update");
        match msg {
            Msg::Navbar(msg) => self.navbar.update(msg, &mut orders.proxy(Msg::Navbar)),
            Msg::Websocket(msg) => {
                self.websocket.update(msg, &mut orders.proxy(Msg::Websocket));
            }
        }
    }
}
pub fn get_host() -> Result<String, JsValue> {
    web_sys::window().unwrap().location().host()
}

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_tracing();
    App::start(
        "app",
        Root::init,
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
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
