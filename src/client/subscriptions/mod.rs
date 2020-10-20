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
        PriceSubscription,
        PriceHistoryRequest,
        ServerMessage,
        ClientMessage,
    },
};
use tracing::debug;
use rql::*;

#[derive(Debug)]
pub struct SubscriptionList {
    subscriptions: HashMap<Id<PriceSubscription>, chart::SubscriptionChart>,
    server_msg_sub: SubHandle,
}
impl SubscriptionList {
    fn add_subscription_request() -> ClientMessage {
        ClientMessage::AddPriceSubscription(
            PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: None,
                paginator: None,
            }
        )
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetList,
    SetList(HashMap<Id<PriceSubscription>, PriceSubscription>),
    Subscription(Id<PriceSubscription>, chart::Msg),
    AddSubscription,
}
impl Init<()> for SubscriptionList {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.send_msg(Msg::GetList);
        Self {
            server_msg_sub: orders.subscribe_with_handle(|msg: ServerMessage| {
                debug!("Received Server Message");
                match msg {
                    ServerMessage::SubscriptionList(list) => Some(Msg::SetList(list)),
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
                orders.notify(ClientMessage::GetPriceSubscriptionList);
            },
            Msg::SetList(list) => {
                debug!("Setting SubscriptionList");
                self.subscriptions = list.into_iter().map(|(id, sub)| {
                    (
                        id.clone(),
                        chart::SubscriptionChart::init(
                            sub,
                            &mut orders.proxy(move |msg| Msg::Subscription(id, msg))
                        )
                    )
                })
                .collect();
                orders.notify(ClientMessage::AddPriceSubscription(PriceHistoryRequest::from("SOLBTC".to_string())));
            },
            Msg::Subscription(id, msg) => {
                if let Some(subscription) = self.subscriptions.get_mut(&id) {
                    subscription.update(msg, &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg)));
                } else {
                    error!("Subscription {} not found!", id);
                }
            }
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
