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
    props: Transactions<C>,
}
impl<C: 'static + Currency> From<Transactions<C>> for TransactionsView<C> {
    fn from(transactions: Transactions<C>) -> Self {
        Self {
            props: transactions,
        }
    }
}
impl<C: 'static + Currency> Component for TransactionsView<C> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            props: Vec::new().into()
        }
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        html!{
            <div class="transaction-table">
                <div class="transaction-table-header">
                    <div class="transaction-header">{"Date"}</div>
                    <div class="transaction-header">{"Amount"}</div>
                    <div class="transaction-header">{"Partner"}</div>
                    <div class="transaction-header">{"Purposes"}</div>
                </div>
                <div class="transaction-table-body">
                    {for self.props.iter().map(|t| TransactionView::from(t.clone()).view())}
                </div>
            </div>
        }
    }
}
