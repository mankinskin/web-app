use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use rql::{
    Id,
};
use crate::{
    root::{
        GMsg,
    },
    fetch::{
        self,
    },
    tasks::*,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub task_id: Id<Task>,
    pub task: Option<Task>,
}
impl Model {
    pub fn preview(&self) -> preview::Model {
        preview::Model::from(self.clone())
    }
    fn ready(id: Id<Task>, task: Task) -> Self {
        Self {
            task_id: id,
            task: Some(task),
        }
    }
}
impl From<&Entry<Task>> for Model {
    fn from(entry: &Entry<Task>) -> Self {
        Self::ready(*entry.id(), entry.data().clone())
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self::from(&entry)
    }
}
impl From<Id<Task>> for Model {
    fn from(id: Id<Task>) -> Self {
        Self {
            task_id: id,
            task: None,
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Fetch(fetch::Msg<Task>),
}
impl From<fetch::Msg<Task>> for Msg {
    fn from(msg: fetch::Msg<Task>) -> Self {
        Msg::Fetch(msg)
    }
}
impl Msg {
    pub fn fetch_task(id: Id<Task>) -> Msg {
        Msg::Fetch(fetch::Msg::Request(fetch::Request::Get(Query::Id(id))))
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Fetch(msg) => {
            match msg {
                fetch::Msg::Request(request) => {
                    orders.perform_cmd(
                        fetch::fetch(
                            url::Url::parse("http://localhost:8000/api/tasks").unwrap(),
                            request
                        )
                        .map(|msg| Msg::from(msg))
                    );
                },
                fetch::Msg::Response(response) => {
                    match response {
                        fetch::Response::Get(data) => {
                            model.task = Some(data);
                        },
                        _ => {}
                    }
                },
                fetch::Msg::Error(error) => {
                    seed::log(error);
                },
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.task {
        Some(model) => {
            div![
                h1!["Task"],
                p![model.title()],
            ]
        },
        None => {
            div![
                h1!["Task"],
                p!["Loading..."],
            ]
        },
    }
}
