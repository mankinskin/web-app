use components::{
    Component,
    Init,
    Viewable,
};
pub mod chart;
use seed::{
    prelude::*,
    *,
};
use std::collections::HashMap;
use crate::{
    shared::{
        subscription::{
            Request,
            Response,
        },
        PriceSubscription,
        PriceHistoryRequest,
        ClientMessage,
    },
};
use database_table::Entry;
use tracing::debug;
use rql::*;

#[derive(Debug)]
pub struct SubscriptionList {
    subscriptions: HashMap<Id<PriceSubscription>, chart::SubscriptionChart>,
    server_msg_sub: SubHandle,
}
impl SubscriptionList {
    fn add_subscription_request() -> ClientMessage {
        ClientMessage::Subscriptions(Request::AddPriceSubscription(
            PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: None,
                paginator: None,
            }
        ))
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetList,
    SetList(Vec<Entry<PriceSubscription>>),
    Subscription(Id<PriceSubscription>, chart::Msg),
    AddSubscription,
}
impl Init<()> for SubscriptionList {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.send_msg(Msg::GetList);
        Self {
            server_msg_sub: orders.subscribe_with_handle(|msg: Response| {
                debug!("Received Subscription Response");
                match msg {
                    Response::SubscriptionList(list) => Some(Msg::SetList(list)),
                    Response::PriceHistory(id, history) => Some(Msg::Subscription(id, chart::Msg::AppendCandles(history.candles))),
                    _ => None,
                }
            }),
            subscriptions: HashMap::new(),
        }
    }
}
impl Component for SubscriptionList {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::GetList => {
                orders.notify(ClientMessage::Subscriptions(Request::GetPriceSubscriptionList));
            },
            Msg::SetList(list) => {
                debug!("Setting SubscriptionList");
                self.subscriptions = list.into_iter().map(|entry| {
                    let id = entry.id.clone();
                    orders.notify(ClientMessage::Subscriptions(
                        Request::GetHistoryUpdates(id.clone())
                    ));
                    (
                        id.clone(),
                        chart::SubscriptionChart::init(
                            entry.data,
                            &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg))
                        )
                    )
                })
                .collect();
            },
            Msg::Subscription(id, msg) => {
                if let Some(subscription) = self.subscriptions.get_mut(&id) {
                    subscription.update(msg, &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg)));
                } else {
                    error!("Subscription {} not found!", id);
                }
            },
            Msg::AddSubscription => {
                // TODO do this over http maybe
                // TODO identify responses over websocket
                orders.notify(Self::add_subscription_request());
            }
        }
    }
}
impl Viewable for SubscriptionList {
    fn view(&self) -> Node<Msg> {
        let list = self.subscriptions
                .iter()
                .map(move |(id, chart)| {
                    (id.clone(), chart.view())
                })
                .collect::<Vec<_>>();
        div![
            list.into_iter().map(|(id, item)| item.map_msg(move |msg| Msg::Subscription(id, msg)))
        ]
    }
}
