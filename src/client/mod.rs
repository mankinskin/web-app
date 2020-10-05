pub mod chart;
pub mod page;
pub mod router;
pub mod websocket;

use components::{
    Component,
    Init,
    Viewable,
};
use router::Router;
use seed::{
    prelude::*,
    *,
};
use tracing::debug;

fn init_tracing() {
    tracing_wasm::set_as_global_default();
    debug!("Tracing initialized.");
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
        |url, orders| Router::init(url, orders),
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
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
