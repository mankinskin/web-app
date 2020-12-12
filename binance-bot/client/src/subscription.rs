use components::{
    Component,
    Init,
    Viewable,
    Editor,
    Previewable,
};
use seed::{
    prelude::*,
    *,
};
use crate::{
    shared::{
        subscriptions::{
            PriceSubscription,
        },
        Route,
    },
    subscriptions::{
        chart::{
            self,
            SubscriptionChart,
        },
    },
};
use database_table::{
    Entry,
    RemoteTable,
    TableRoutable,
};
use async_trait::async_trait;
use rql::*;
use std::result::Result;

#[derive(Debug)]
pub struct SubscriptionInfo {
    subscription: PriceSubscription,
    chart: SubscriptionChart,
    editor: Option<Editor<PriceSubscription>>,
}
impl TableRoutable<PriceSubscription> for SubscriptionInfo {
    type Route = Route;
    fn table_route() -> Self::Route {
        PriceSubscription::table_route()
    }
    fn entry_route(id: Id<PriceSubscription>) -> Self::Route {
        PriceSubscription::entry_route(id)
    }
}
#[async_trait(?Send)]
impl RemoteTable<PriceSubscription> for SubscriptionInfo {
    type Error = <PriceSubscription as RemoteTable>::Error;
    async fn get(id: Id<PriceSubscription>) -> Result<Option<Entry<PriceSubscription>>, Self::Error> {
        PriceSubscription::get(id).await
    }
    async fn delete(id: Id<PriceSubscription>) -> Result<Option<PriceSubscription>, Self::Error> {
        PriceSubscription::delete(id).await
    }
    async fn get_all() -> Result<Vec<Entry<PriceSubscription>>, Self::Error> {
        PriceSubscription::get_all().await
    }
    async fn post(data: PriceSubscription) -> Result<Id<PriceSubscription>, Self::Error> {
        PriceSubscription::post(data).await
    }
}
#[derive(Debug)]
pub enum Msg {
    OpenEditor,
    Editor(<Editor<PriceSubscription> as Component>::Msg),
    Chart(chart::Msg),
}
impl Init<Entry<PriceSubscription>> for SubscriptionInfo {
    fn init(entry: Entry<PriceSubscription>, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            subscription: entry.data().clone(),
            chart: SubscriptionChart::init(entry, &mut orders.proxy(Msg::Chart)),
            editor: None,    
        }
    }
}
impl Component for SubscriptionInfo {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::OpenEditor => {
                self.editor = Some(Editor::default());
            }
            Msg::Editor(msg) => {
                if let Some(ed) = &mut self.editor {
                    let new = match msg {
                        editor::Msg::Cancel => Some(None),
                        editor::Msg::Submit => Some(None),
                        _ => None,
                    };
                    ed.update(msg, &mut orders.proxy(Msg::Editor));
                    if let Some(new) = new {
                        self.editor = new;
                    }
                }
            },
        }
    }
}
impl Viewable for SubscriptionInfo {
    fn view(&self) -> Node<Msg> {
        div![
            format!("{:?}", self.subscription)
        ]
    }
}
impl Previewable for SubscriptionInfo {
    fn preview(&self) -> Node<Msg> {
        div![
            format!("{:?}", self.subscription)
        ]
    }
}
