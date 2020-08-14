extern crate async_tls;
extern crate lazy_static;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate seed;
extern crate console_error_panic_hook;
extern crate components;
use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    Config,
    View,
};

#[wasm_bindgen(start)]
pub fn render() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::start("app",
               |url, orders| Model::default(),
               |msg, model, orders| model.update(msg, orders),
               View::view,
    );
}
#[derive(Clone, Default)]
pub struct Model {
}
#[derive(Clone, Debug)]
pub enum Msg {
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) {
    }
}
impl View for Model {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p!["Hello from Seed!"]
        ]
    }
}
