
use components::{
    Component,
    Init,
    Viewable,
};
use crate::{
    shared::PriceSubscription,
    chart::{
        self,
        Chart,
    },
};
use seed::{
    *,
    prelude::*,
};
#[derive(Debug)]
pub struct SubscriptionChart {
    subscription: PriceSubscription,
    chart: Chart,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Chart(chart::Msg),
}
impl Init<PriceSubscription> for SubscriptionChart {
    fn init(subscription: PriceSubscription, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            chart: Chart::init((), &mut orders.proxy(Msg::Chart)),
            subscription,
        }
    }
}

impl Component for SubscriptionChart {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
    }
}
impl Viewable for SubscriptionChart {
    fn view(&self) -> Node<Msg> {
        div![
            format!("{:#?}", self.subscription),
            self.chart.view().map_msg(Msg::Chart),
        ]
    }
}
