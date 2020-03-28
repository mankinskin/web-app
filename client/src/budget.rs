pub use plans::{
    *,
    currency::*,
};
use yew::{
    *,
};

use crate::{
    *,
    transactions::*,
};

pub enum Msg {
    Update,
}
pub struct BudgetView<C: 'static + Currency> {
    link: ComponentLink<Self>,
    model: Budget<C>,
}

impl<C: 'static + Currency> Component for BudgetView<C> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut b = Budget::create("My Budget", 0);
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        Self {
            link,
            model: b,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div>
                <h1>{self.model.name()}</h1>
                {TransactionsView::from(self.model.transactions.clone()).view()}
            </div>
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use std::sync::{Mutex};
        lazy_static!{
            static ref INDEX: Mutex<u32> = Mutex::new(0);
        }

        match msg {
            Msg::Update => {
                true
            },
            _ => false
        }
    }
}
