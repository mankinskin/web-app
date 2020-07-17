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
use interpreter::{
    text::*,
};
use std::result::Result;
use std::convert::{
    TryFrom,
};

#[derive(Debug,Clone, Default)]
pub struct Model {
    text: String,
}
#[derive(Clone, Debug)]
pub enum Msg {
    SetText(String),
    InterpretText,
    InterpreterResponse(Result<String, String>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::SetText(t) => {
                self.text = t;
            },
            Msg::InterpretText => {
                let text = Text::try_from(self.text.as_str()).unwrap();
                orders.perform_cmd(
                    api::interpret_text(text)
                        .map(|result: Result<String, FetchError>| {
                            Msg::InterpreterResponse(result.map_err(|e| format!("{:?}", e)))
                        })
                );
            },
            Msg::InterpreterResponse(result) => {
                match result {
                    Ok(s) => {
                        log!(s);
                    },
                    Err(e) => {
                        log!(e)
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
            ]
        ]
    }
}
