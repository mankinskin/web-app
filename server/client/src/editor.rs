use seed::{
    prelude::*,
};
use plans::{
    task::*,
};
use crate::{
    config::{
        Component,
        View,
        Config,
    },
    root::{
        self,
        GMsg,
    },
    tasks::{*},
};
use std::result::Result;

#[derive(Clone)]
pub enum Model<T> {
    Entry(entry::Model<T>),
    New(newdata::Model<T>),
}
impl<T> Config<Model<T>> for entry::Model<T> {
    fn into_model(self, _orders: &mut impl Orders<Msg<T>, root::GMsg>) -> Model<T> {
        Model {
            data: self.data.unwrap_or(Default::default()),
            id: Some(self.id),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>, root::GMsg>) {
    }
}
impl From<Entry<Task>> for Model {
    fn from(entry: Entry<Task>) -> Self {
        Self {
            task_id: Some(entry.id().clone()),
            task: entry.data().clone(),
            ..Default::default()
        }
    }
}
#[derive(Clone)]
pub enum Msg<T> {
    Cancel,
    Submit,
    //Created(Result<Id<Task>, String>),
}
impl<T> Component for Model<T> {
    type Msg = Msg<T>;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            Msg::Cancel => {},
            Msg::Submit => {},
            //    let task = model.task.clone();
            //    if let Some(id) = model.project_id {
            //        orders.perform_cmd(
            //            api::project_create_subtask(id, task)
            //                .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
            //        );
            //    } else {
            //        orders.perform_cmd(
            //            api::post_task(task)
            //                .map(|res| Msg::Created(res.map_err(|e| format!("{:?}", e))))
            //        );
            //    }
            //},
            //Msg::Created(res) => {
            //    match res {
            //        Ok(id) => model.task_id = Some(id),
            //        Err(e) => { seed::log(e); },
            //    }
            //},
        }
    }
}

impl<T> View for Model<T> {
    fn view(&self) -> Node<Msg> {
        form![
            style!{
                St::Display => "grid",
                St::GridTemplateColumns => "1fr",
                St::GridGap => "10px",
                St::MaxWidth => "20%",
            },
            if let Some(_) = model.task_id {
                h1!["Edit Task"]
            } else {
                h1!["New Task"]
            },
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
            // Submit Button
            button![
                attrs!{
                    At::Type => "submit",
                },
                "Create"
            ],
            ev(Ev::Submit, |ev| {
                ev.prevent_default();
                Msg::Submit
            }),
            // Cancel Button
            button![simple_ev(Ev::Click, Msg::Cancel), "Cancel"],
        ]
    }
}
