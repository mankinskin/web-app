use yew::{
    *,
};

use plans::{
    *,
    currency::*,
};
use crate::{
    *,
    transaction::*,
};

pub struct TransactionsView<C: 'static + Currency> {
    model: Transactions<C>,
}
impl<C: 'static + Currency> From<Transactions<C>> for TransactionsView<C> {
    fn from(transactions: Transactions<C>) -> Self {
        Self {
            model: transactions,
        }
    }
}
impl<C: 'static + Currency> Component for TransactionsView<C> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            model: Vec::new().into()
        }
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        html!{
            <table class="transaction-table">
                <caption class="transaction-caption">{"Your Transactions"}</caption>
                <tr class="transaction-row">
                    <th class="transaction-header">{"Date"}</th>
                    <th class="transaction-header">{"Amount"}</th>
                    <th class="transaction-header">{"Partner"}</th>
                    <th class="transaction-header">{"Purposes"}</th>
                </tr>
                {for self.model.iter().map(|t| TransactionView::from(t.clone()).view())}
            </table>
        }
    }
}
