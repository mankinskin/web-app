use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
};
use crate::{
    preview::{
        Preview,
    },
    editor::{
        Edit,
    },
    entry::{
        self,
    },
    config::{
        Component,
        View,
    },
};

pub mod editor;
pub mod list;
pub mod profile;


#[derive(Clone)]
pub enum Msg {
    SetName(String),
    SetDescription(String),
    Entry(Box<entry::Msg<Project>>),
}
impl Component for Project {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::SetName(n) => {
                self.set_name(n);
            },
            Msg::SetDescription(d) => {
                self.set_description(d);
            },
            Msg::Entry(_) => {}
        }
    }
}
impl View for Project {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p![self.name()],
        ]
    }
}
impl Preview for Project {
    fn preview(&self) -> Node<Msg> {
        div![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
                St::Cursor => "pointer",
            },
            h3![
                style!{
                    St::Margin => "0",
                },
                self.name(),
            ],
            div![],

            p![
                style!{
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.tasks().len(),

            p![
                style!{
                    St::Margin => "0",
                },
                "Members:"
            ],
            self.members().len(),

            button![
                ev(Ev::Click, |_| Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
impl Edit for Project {
    fn edit(&self) -> Node<Msg> {
        form![
            label![
                "Name"
            ],
            input![
                attrs!{
                    At::Placeholder => "Name",
                    At::Value => self.name(),
                },
                //input_ev(Ev::Input, Msg::SetName)
            ],
            label![
                "Description"
            ],
            textarea![
                attrs!{
                    At::Placeholder => "Description...",
                    At::Value => self.description(),
                },
                //input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
