extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate http;
extern crate anyhow;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate rql;
extern crate plans;
extern crate budget;
extern crate updatable;
extern crate database;

use seed::{
    *,
    prelude::*,
};
pub mod login;

struct Model {
    login: login::Model,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        Self {
            login: login::Model::default(),
        }
    }
}

#[derive(Clone)]
enum Msg {
    Login(login::Msg),
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(msg) => {
            login::update(msg.clone(), &mut model.login, &mut orders.proxy(Msg::Login));
        },
    }
}

/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {

    // Attrs, Style, Events, and children may be defined separately.
    div![
        login::view(&model.login)
            .map_msg(Msg::Login),
    ]
}


#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .build_and_start();
}
