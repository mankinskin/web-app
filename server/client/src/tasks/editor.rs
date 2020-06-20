use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use crate::{
    root::{
        GMsg,
    },
    tasks::*,
    fetch,
    fetch::Request,
};

#[derive(Clone)]
pub struct Model {
    pub task: Task,
}
impl Model {
    fn empty() -> Self {
        Self {
            task: Task::new(String::new()),
        }
    }
}
impl Default for Model {
    fn default() -> Self {
        Self::empty()
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeTitle(String),
    ChangeDescription(String),
    Create,
    Cancel,
    Fetch(fetch::Msg<Task>)
}
impl From<fetch::Msg<Task>> for Msg {
    fn from(msg: fetch::Msg<Task>) -> Self {
        Msg::Fetch(msg)
    }
}
impl Msg {
    pub fn post_task(task: &Task) -> Msg {
        Msg::Fetch(fetch::Msg::Request(Request::Post(task.clone())))
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeTitle(n) => {
            model.task.set_title(n);
        },
        Msg::ChangeDescription(d) => {
            model.task.set_description(d);
        },
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/tasks").unwrap(),
                            request,
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {
                    seed::log(error);
                },
            }
        },
        Msg::Create => {
            orders.send_msg(Msg::post_task(&model.task));
        },
        Msg::Cancel => {},
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    form![
        h1!["New Task"],
        label![
            "Title"
        ],
        input![
            attrs!{
                At::Placeholder => "Title",
                At::Value => model.task.title(),
            },
            input_ev(Ev::Input, Msg::ChangeTitle)
        ],
        label![
            "Description"
        ],
        textarea![
            attrs!{
                At::Placeholder => "Description...",
                At::Value => model.task.description(),
            },
            input_ev(Ev::Input, Msg::ChangeDescription)
        ],
        // Cancel Button
        button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
        // Create Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Create"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Create
        }),
    ]
}
