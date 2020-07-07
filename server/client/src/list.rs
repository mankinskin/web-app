use seed::{
    *,
    prelude::*,
};
use crate::{
    config::{
        Component,
        View,
        Config,
    },
    root::{
        GMsg,
    },
    preview::{self, Preview},
    entry::{
        self,
        TableItem,
    },
};
use database::{
    Entry,
};
use std::result::Result;

#[derive(Clone, Default)]
pub struct Model<T: TableItem>
{
    previews: Vec<preview::Model<T>>,
}
impl<T: Component + TableItem + Default> Config<Model<T>> for Msg<T>
{
    fn into_model(self, _orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model::default()
    }
    fn send_msg(self, orders: &mut impl Orders<Msg<T>, GMsg>) {
        orders.send_msg(self);
    }
}
impl<T: Component + TableItem> Config<Model<T>> for Vec<Entry<T>>
{
    fn into_model(self, orders: &mut impl Orders<Msg<T>, GMsg>) -> Model<T> {
        Model {
            previews: init_previews(self, orders),
        }
    }
    fn send_msg(self, _orders: &mut impl Orders<Msg<T>, GMsg>) {
    }
}
fn init_previews<T: Component + TableItem>(entries: Vec<Entry<T>>, orders: &mut impl Orders<Msg<T>, GMsg>) -> Vec<preview::Model<T>>
{
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)|
            Config::init(
                entry.clone(),
                &mut orders
                    .proxy(move |msg| Msg::Preview(i, msg))
            )
        )
        .collect()
}
#[derive(Clone)]
pub enum Msg<T: Component + TableItem>
{
    GetAll,
    All(Result<Vec<Entry<T>>, String>),

    Preview(usize, preview::Msg<T>),

    //OpenEditor,
    //Editor(editor::Msg),
}
impl<T: Component + TableItem> Component for Model<T>
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Msg<T>, GMsg>) {
        match msg {
            Msg::GetAll => {
                orders.perform_cmd(
                    T::get_all()
                        .map(|res| Msg::All(res))
                );
            },
            Msg::All(res) => {
                match res {
                    Ok(entries) => self.previews = init_previews(entries, orders),
                    Err(e) => { seed::log(e); },
                }
            },
            Msg::Preview(index, msg) => {
                self.previews[index].update(
                    msg.clone(),
                    &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg))
                );
                if let preview::Msg::Entry(entry::Msg::Deleted(_)) = msg {
                    self.previews.remove(index);
                }
            },
            //Msg::OpenEditor => {
            //    self.editor = match self.project_id {
            //        Some(id) => {
            //            Some(Config::init(id, &mut orders.proxy(Msg::Editor)))
            //        },
            //        None => {
            //            Some(editor::Model::default())
            //        },
            //    };
            //},
            //Msg::Editor(msg) => {
            //    if let Some(editor) = &mut self.editor {
            //        editor::update(
            //            msg.clone(),
            //            editor,
            //            &mut orders.proxy(Msg::Editor)
            //        );
            //    }
            //    match msg {
            //        editor::Msg::Cancel => {
            //            self.editor = None;
            //        },
            //        editor::Msg::Created(_) => {
            //            orders.send_msg(
            //                if let Some(id) = self.project_id {
            //                    Msg::GetProjectTasks(id)
            //                } else {
            //                    Msg::GetAll
            //                }
            //            );
            //        },
            //        _ => {},
            //    }
            //},
        }
    }
}
impl <T: Component + Preview + TableItem> View for Model<T>
{
    fn view(&self) -> Node<Msg<T>> {
        div![
            //if let Some(model) = &model.editor {
            //    editor::view(&model).map_msg(Msg::Editor)
            //} else {
            //    if let Some(_) = api::auth::get_session() {
            //        button![
            //            simple_ev(Ev::Click, Msg::OpenEditor),
            //            "New Task"
            //        ]
            //    } else { empty![] }
            //},

            ul![
                self.previews.iter().enumerate()
                    .map(|(i, preview)| li![
                         preview.view()
                            .map_msg(move |msg| Msg::Preview(i.clone(), msg))
                    ])
            ]
        ]
    }
}
