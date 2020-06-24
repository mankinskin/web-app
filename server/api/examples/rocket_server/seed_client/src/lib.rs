#![feature(async_closure)]
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate wasm_bindgen_futures;
extern crate url;
extern crate wasm_bindgen;
extern crate seed;
extern crate plans;
#[macro_use] extern crate lazy_static;

mod storage;

use plans::{
    user::*,
};
use seed::{
    *,
    prelude::*,
};
use std::sync::{
    Mutex,
};

lazy_static! {
    static ref USER_SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
}
pub fn set_session(session: UserSession) {
    *USER_SESSION.lock().unwrap() = Some(session);
}
pub fn get_session() -> Option<UserSession> {
    USER_SESSION.lock().unwrap().clone()
}
pub fn end_session() {
    *USER_SESSION.lock().unwrap() = None;
}
#[derive(Clone, Default)]
pub struct Model {
    a: u32,
    b: u32,
    result: Option<Result<u32, String>>,
}
#[derive(Clone)]
pub enum Msg {
    Call,
    CallResult(Result<u32, String>),
    ChangeA(String),
    ChangeB(String),
}
#[derive(Clone)]
pub enum GMsg {
    Root(Msg),
    SetSession(UserSession),
    EndSession,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Call => {
            seed::log(format!("calling add({}, {})...", model.a, model.b));
            orders.perform_cmd(
                api::add(model.a, model.b)
                    .map(|res|
                         res.map_err(|e| format!("{:?}", e))
                    )
                    .map(Msg::CallResult)
            );
        },
        Msg::CallResult(res) => {
            seed::log(format!("got result: {:?}", res));
            model.result = Some(res);
        },
        Msg::ChangeA(s) => {
            match s.parse::<u32>() {
                Ok(a) => model.a = a,
                Err(e) => model.result = Some(Err(e.to_string())),
            }
        },
        Msg::ChangeB(s) => {
            match s.parse::<u32>() {
                Ok(b) => model.b = b,
                Err(e) => model.result = Some(Err(e.to_string())),
            }
        },
    }
}
pub fn sink(msg: GMsg, _model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        GMsg::Root(msg) => {
            orders.send_msg(msg);
        },
        GMsg::SetSession(session) => {
            seed::log!("Setting session...");
            set_session(session.clone());
            storage::store_session(&session.clone());
            //orders.send_msg(Msg::Call);
        },
        GMsg::EndSession => {
            seed::log!("ending session");
            storage::clean_storage();
            end_session()
        },
    }
}
pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    form![
        p![
            "add("
        ],
        label![
            "A:"
        ],
        input![
            attrs!{
                At::Placeholder => "Number",
                At::Value => model.a,
            },
            input_ev(Ev::Input, Msg::ChangeA)
        ],
        div![
            model.a
        ],
        p![
            ","
        ],
        label![
            "B:"
        ],
        input![
            attrs!{
                At::Placeholder => "Number",
                At::Value => model.b,
            },
            input_ev(Ev::Input, Msg::ChangeB)
        ],
        div![
            model.b
        ],
        p![
            ")"
        ],
        // Login Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Call!"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Call
        }),
        if let Some(result) = &model.result {
            empty![]
        } else {
            empty![]
        }
    ]
}
#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .sink(sink)
        .build_and_start();
}
