use seed::{
    prelude::*,
};
use plans::{
    project::*,
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
    projects::*,
    tasks,
};
use database::{
    Entry,
};

impl Component for Model {
    type Msg = Msg;
}
impl Config<Model> for Id<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        Model {
            project: Config::init(self.clone(), &mut orders.proxy(Msg::Project)),
            tasks: Config::init(self, &mut orders.proxy(Msg::TaskList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
impl Config<Model> for Entry<Project> {
    fn into_model(self, orders: &mut impl Orders<Msg, root::GMsg>) -> Model {
        let id = self.id().clone();
        Model {
            project: Config::init(self, &mut orders.proxy(Msg::Project)),
            tasks: Config::init(id, &mut orders.proxy(Msg::TaskList)),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg, root::GMsg>) {
    }
}
#[derive(Clone)]
pub struct Model {
    pub project: project::Model,
    pub tasks: tasks::list::Model,
}
#[derive(Clone)]
pub enum Msg {
    Project(project::Msg),
    TaskList(tasks::list::Msg),
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Project(msg) => {
            project::update(
                msg,
                &mut model.project,
                &mut orders.proxy(Msg::Project)
            );
        },
        Msg::TaskList(msg) => {
            tasks::list::update(
                msg,
                &mut model.tasks,
                &mut orders.proxy(Msg::TaskList)
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    div![
        project::view(&model.project).map_msg(Msg::Project),
        tasks::list::view(&model.tasks).map_msg(Msg::TaskList),
    ]
}
