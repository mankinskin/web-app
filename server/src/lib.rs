extern crate yew;
extern crate stdweb;
extern crate wasm_bindgen;
#[macro_use] extern crate lazy_static;
extern crate plans;

mod transaction;
mod transactions;
mod budget;

use wasm_bindgen::prelude::*;
use plans::{
    currency::*,
};

pub enum Msg {
    PrevImage,
    NextImage,
    Init,
}
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    //let mut budget = Budget::create("My Budget", Euro(140));
    //budget.get(Euro(19)).set_partner("Papa".into());
    //budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
    //budget.give(Euro(49)).set_purpose("Fahrstunde".into());
    //budget.give(Euro(19)).set_purpose("Essen".into()).set_partner("Papa".into());
    //println!("{}", budget);

    //interpreter::run().unwrap();
    //console!(log, "yew::initialize");
    yew::initialize();

    //let document = web_sys::window().unwrap().document().unwrap();
    //let head = document.head().unwrap();
    //head.set_inner_text("<head>
    //  <link rel=\"stylesheet\" href=\"node_modules/xterm/css/xterm.css\" />
    //  <script src=\"node_modules/xterm/lib/xterm.js\"></script>
    //  <style>
    //      body {
    //          background-color: rgb(20, 20, 20);
    //      }
    //      h1, caption, table, td {
    //          font-family: Impact, Charcoal, sans-serif;
    //          color: lightblue;
    //      }
    //      table, th, td {
    //          border-bottom: 1px solid lightgray;
    //          border-collapse: collapse;
    //          padding: 10px;
    //      }
    //      .transaction {
    //          color: blue;
    //      }
    //      .flex-container {
    //          display: flex;
    //          flex-direction: row;
    //      }
    //      caption {
    //          font-weight: bold;
    //      }
    //  </style>
    //</head>");

    //console!(log, "App::new");
    yew::App::<budget::BudgetView<Euro>>::new()
        .mount_to_body()
        .send_message(Msg::Init);
    //console!(log, "run_loop");
    yew::run_loop();
    Ok(())
}
