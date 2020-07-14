use seed::{
    *,
    prelude::*,
};
use plans::{
    task::*,
};
use rql::{
    *,
};
use crate::{
    preview::{
        Preview,
    },
    entry::{
        self,
    },
    config::{
        Component,
        View,
    },
    editor::{
        Edit,
    },
};

pub mod editor;
pub mod profile;
pub mod list;

#[derive(Clone)]
pub enum Msg {
    SetDescription(String),
    SetTitle(String),
    Entry(Box<entry::Msg<Task>>),
}
impl Component for Task {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, _orders: &mut impl Orders<Msg>) {
        match msg {
            Msg::SetTitle(n) => {
                self.set_title(n);
            },
            Msg::SetDescription(d) => {
                self.set_description(d);
            },
            Msg::Entry(_) => {},
        }
    }
}
impl View for Task {
    fn view(&self) -> Node<Self::Msg> {
        div![
            p![self.title()],
        ]
    }
}
impl Preview for Task {
    fn preview(&self) -> Node<Msg> {
        div![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr 1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
                St::Cursor => "pointer",
            },
            //ev(Ev::Click, Msg::Entry(Box::new(entry::Msg::Preview(Box::new(preview::Msg::Open))))),
            h3![
                style!{
                    St::Margin => "0",
                },
                self.title(),
            ],
            div![],
            p![
                style!{
                    St::Margin => "0",
                },
                "Subtasks:"
            ],
            self.subtasks().len(),
            p![
                style!{
                    St::Margin => "0",
                },
                "Assignees:"
            ],
            self.assignees().len(),
            button![
                ev(Ev::Click, |_| Msg::Entry(Box::new(entry::Msg::Delete))),
                "Delete"
            ],
        ]
    }
}
impl Edit for Task {
    fn edit(&self) -> Node<Msg> {
        form![
            label![
                "Title"
            ],
            input![
                attrs!{
                    At::Placeholder => "Title",
                    At::Value => self.title(),
                },
                input_ev(Ev::Input, Msg::SetTitle)
            ],
            label![
                "Description"
            ],
            textarea![
                attrs!{
                    At::Placeholder => "Description...",
                    At::Value => self.description(),
                },
                input_ev(Ev::Input, Msg::SetDescription)
            ],
        ]
    }
}
