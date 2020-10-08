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
    websocket,
    shared::{
        PriceSubscription,
        ServerMessage,
        ClientMessage,
    },
};
use tracing::debug;

#[derive(Debug)]
pub struct SubscriptionList {
    subscriptions: HashMap<usize, chart::SubscriptionChart>,
    server_msg_sub: SubHandle,
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetList,
    SetList(HashMap<usize, PriceSubscription>),
    Subscription(usize, chart::Msg)
}
impl Init<()> for SubscriptionList {
    fn init(_: (), orders: &mut impl Orders<Msg>) -> Self {
        orders.notify(ClientMessage::GetPriceSubscriptionList);
        let server_msg_sub = orders.subscribe_with_handle(|msg: ServerMessage| {
            debug!("Received Server Message");
            match msg {
                ServerMessage::SubscriptionList(list) => Some(Msg::SetList(list)),
                _ => None,
            }
        });
        Self {
            server_msg_sub,
            subscriptions: HashMap::new(),
        }
    }
}
impl Component for SubscriptionList {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::GetList => {
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

            },
            Msg::Subscription(id, msg) => {
                if let Some(subscription) = self.subscriptions.get_mut(&id) {
                    subscription.update(msg, &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg)));
                } else {
                    error!("Subscription {} not found!", id);
                }
                    
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
