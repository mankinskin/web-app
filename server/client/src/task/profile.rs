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
    config::{
        Component,
        View,
        Config,
    },
    root::{
        self,
        GMsg,
    },
    entry,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub entry: entry::Model<Task>,
    //pub editor: Option<editor::Model>,
}
impl Config<Model> for Id<Task> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            entry: Config::init(self.clone(), &mut orders.proxy(Msg::Entry)),
            //editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<Task> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            entry: Config::init(self, &mut orders.proxy(Msg::Entry)),
            //editor: None,
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    //Edit,
    //Editor(editor::Msg),

    Entry(entry::Msg<Task>),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg, GMsg>) {
        match msg {
            //Msg::Edit => {
            //    model.editor = Some(Config::init(model.task.clone(), &mut orders.proxy(Msg::Editor)));
            //},
            //Msg::Editor(msg) => {
            //    if let Some(model) = &mut model.editor {
            //        editor::update(
            //            msg,
            //            model,
            //            &mut orders.proxy(Msg::Editor),
            //        );
            //    }
            //},
            Msg::Entry(msg) => {
                self.entry.update(
                    msg,
                    &mut orders.proxy(Msg::Entry),
                );
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        //if let Some(model) = &self.editor {
        //    div![
        //        editor::view(&model)
        //            .map_msg(Msg::Editor)
        //    ]
        //} else {
            self.entry.view()
                .map_msg(Msg::Entry)
        //}
    }
}
