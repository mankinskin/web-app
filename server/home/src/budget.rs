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

pub struct BudgetView<C: 'static + Currency> {
    model: Budget<C>,
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
            model: b,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="budget-container">
                <div class="budget-header">
                    <div class="budget-name">{
                        self.model.name()
                    }</div>
                </div>
                {TransactionsView::from(self.model.transactions.clone()).view()}
            </div>
        }
    }
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}
