pub use plans::{
    *,
};
use yew::{
    *,
};

use budget::{
    *,
    currency::*,
};
use crate::{
    *,
    transactions::*,
};

pub struct BudgetView<C: 'static + Currency> {
    props: Budget<C>,
}

impl<C: 'static + Currency> Component for BudgetView<C> {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let mut b = Budget::create("My Budget", 0);
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        Self {
            props: b,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="budget-container">
                <div class="budget-header">
                    <div class="budget-name">{
                        self.props.name()
                    }</div>
                </div>
                {TransactionsView::from(self.props.transactions.clone()).view()}
            </div>
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}
