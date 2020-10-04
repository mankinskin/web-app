use seed::{
    *,
    prelude::*,
};
use components::{
    Component,
    Viewable,
};
use std::result::Result;
use seqraph::{
    NodeInfo,
    mapping::{
        Sequenced,
    },
};

#[derive(Debug,Clone, Default)]
pub struct Model {
    text: String,
    interpreter_response: String,
    query: String,
    query_response: Option<NodeInfo<char>>,
}
#[derive(Clone, Debug)]
pub enum Msg {
    SetText(String),
    InterpretText,
    InterpreterResponse(Result<String, String>),
    SetQuery(String),
    Query,
    QueryResponse(Result<Option<NodeInfo<char>>, String>),
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
                self.query_response = None;
                orders.perform_cmd(
                    api::query_text(self.query.clone())
                        .map(|result: Result<_, FetchError>| {
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
                    }
                }
            },
        }
    }
}
fn view_distance_groups(groups: &Vec<Vec<Sequenced<char>>>) -> Node<Msg> {
    div![
        style!{
            St::Display => "grid",
            St::GridTemplateColumns => "min-content",
            St::GridTemplateRows => "max-content",
            St::GridGap => "20px",
        },
        groups.iter().map(view_distance_group)
    ]
}
fn view_distance_group(group: &Vec<Sequenced<char>>) -> Node<Msg> {
    div![
        style!{
            St::GridRow => "1",
            St::AlignItems => "center",
        },
        group.iter().map(|elem|
            p![elem.to_string()]
        )
    ]
}
fn view_node_info(info: &Option<NodeInfo<char>>) -> Node<Msg> {
    if let Some(info) = info {
        div![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr 1fr",
            },
            div![
                view_distance_groups(&info.incoming_groups),
            ],
            div![
                info.element.to_string()
            ],
            div![
                style!{
                    St::Float => "left",
                },
                view_distance_groups(&info.outgoing_groups),
            ],
        ]
    } else {
        empty![]
    }
}
impl Viewable for Model {
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
                view_node_info(&self.query_response)
            ],
        ]
    }
}
