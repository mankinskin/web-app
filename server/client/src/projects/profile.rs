use seed::{
    *,
    prelude::*,
};
use plans::{
    project::*,
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
    tasks,
    entry,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub entry: entry::Model<Project>,
    pub tasks: tasks::list::Model,
    //pub editor: Option<editor::Model>,
}
impl Config<Model> for Id<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            entry: Config::init(self.clone(), &mut orders.proxy(Msg::Entry)),
            tasks: Config::init(self, &mut orders.proxy(Msg::TaskList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        let id = self.id().clone();
        Config::init(id, orders)
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub enum Msg {
    Entry(entry::Msg<Project>),
    TaskList(tasks::list::Msg),
}
impl Component for Model {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg, GMsg>) {
        match msg {
            Msg::Entry(msg) => {
                self.entry.update(
                    msg,
                    &mut orders.proxy(Msg::Entry)
                );
            },
            Msg::TaskList(msg) => {
                self.tasks.update(
                    msg,
                    &mut orders.proxy(Msg::TaskList)
                );
            },
        }
    }
}
impl View for Model {
    fn view(&self) -> Node<Msg> {
        div![
            self.entry.view()
                .map_msg(Msg::Entry),
            self.tasks.view()
                .map_msg(Msg::TaskList),
        ]
    }
}
