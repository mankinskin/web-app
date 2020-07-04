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
    config::*,
    root::{
        self,
        GMsg,
    },
    tasks::*,
};
use database::{
    Entry,
};

impl Component for Model {
    type Msg = Msg;
}
#[derive(Clone)]
pub struct Model {
    pub task: task::Model,
    pub editor: Option<editor::Model>,
}
impl Config<Model> for Id<Task> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: Config::init(self.clone(), &mut orders.proxy(Msg::Task)),
            editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<Task> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            task: Config::init(self, &mut orders.proxy(Msg::Task)),
            editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    Edit,
    Editor(editor::Msg),

    Task(task::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Edit => {
            model.editor = Some(Config::init(model.task.clone(), &mut orders.proxy(Msg::Editor)));
        },
        Msg::Editor(msg) => {
            if let Some(model) = &mut model.editor {
                editor::update(
                    msg,
                    model,
                    &mut orders.proxy(Msg::Editor),
                );
            }
        },
        Msg::Task(msg) => {
            task::update(
                msg,
                &mut model.task,
                &mut orders.proxy(Msg::Task),
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    if let Some(model) = &model.editor {
        div![
            editor::view(&model)
                .map_msg(Msg::Editor)
        ]
    } else {
        task::view(&model.task)
            .map_msg(Msg::Task)
    }
}
