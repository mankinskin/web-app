use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
    },
};
use std::result::Result;

#[derive(Debug,Clone, Default)]
pub struct Model {
    text: String,
    interpreter_response: String,
    query: String,
    query_response: String,
}
#[derive(Clone, Debug)]
pub enum Msg {
    SetText(String),
    InterpretText,
    InterpreterResponse(Result<String, String>),
    SetQuery(String),
    Query,
    QueryResponse(Result<String, String>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::SetText(t) => {
                self.text = t;
            },
            Msg::InterpretText => {
                self.interpreter_response.clear();
                orders.perform_cmd(
                    api::interpret_text(self.text.clone())
                        .map(|result: Result<String, FetchError>| {
                            Msg::InterpreterResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::InterpreterResponse(result) => {
                match result {
                    Ok(s) => {
                        log!(s);
                        self.interpreter_response = s;
                    },
                    Err(e) => {
                        log!(e);
                        self.interpreter_response = e;
                    }
                }
            },
            Msg::SetQuery(t) => {
                self.query = t;
            },
            Msg::Query => {
                self.query_response.clear();
                orders.perform_cmd(
                    api::query_text(self.query.clone())
                        .map(|result: Result<String, FetchError>| {
                            Msg::QueryResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::QueryResponse(result) => {
                match result {
                    Ok(s) => {
                        log!(s);
                        self.query_response = s;
                    },
                    Err(e) => {
                        log!(e);
                        self.query_response = e;
                    }
                }
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        ul![
            li![
                "Awesome Stuff"
            ],
            li![
                "Look at this too!"
            ],
            input![
                attrs!{
                    At::Placeholder => "Text",
                    At::Value => self.text,
                },
                input_ev(Ev::Input, Msg::SetText)
            ],
            button![
                ev(Ev::Click, |_| Msg::InterpretText),
                "Interpret"
            ],
            p![self.interpreter_response.clone()],
            input![
                attrs!{
                    At::Placeholder => "Query",
                    At::Value => self.query,
                },
                input_ev(Ev::Input, Msg::SetQuery)
            ],
            button![
                ev(Ev::Click, |_| Msg::Query),
                "Query"
            ],
            p![
                style!{
                    St::WhiteSpace => "pre-wrap",
                },
                self.query_response.clone()
            ],
        ]
    }
}
