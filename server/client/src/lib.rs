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
pub mod register;

#[derive(Default)]
struct Model {
    login: login::Model,
    register: register::Model,
}

#[derive(Clone)]
enum Msg {
    Login(login::Msg),
    Register(register::Msg),
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
/// How we update the model
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(msg) => {
            login::update(msg.clone(), &mut model.login, &mut orders.proxy(Msg::Login));
        },
        Msg::Register(msg) => {
            register::update(msg.clone(), &mut model.register, &mut orders.proxy(Msg::Register));
        },
    }
}

/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    div![
        login::view(&model.login)
            .map_msg(Msg::Login),

        register::view(&model.register)
            .map_msg(Msg::Register),
    ]
}


#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .build_and_start();
}
