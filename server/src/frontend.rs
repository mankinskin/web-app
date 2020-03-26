use plans::{
    *,
    currency::*,
    transaction::Transaction,
};
use yew::{
    *,
};
use stdweb::web::*;

pub enum Msg {
    PrevImage,
    NextImage,
    Init,
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
            Msg::Init => {
                true
            },
            _ => false
        }
    }
}
struct TransactionsView<C: 'static + Currency> {
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
    type Message = Msg;
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
            <div>
                <table class="transaction-table">
                    <caption>{"Your Transactions"}</caption>
                    <tr class="transaction-header">
                    <th>{"Date"}</th>
                    <th>{"Amount"}</th>
                    <th>{"Partner"}</th>
                    <th>{"Purposes"}</th>
                    </tr>
                    {for self.model.iter().map(|t| TransactionView::from(t.clone()).view())}
                </table>
            </div>
        }
    }
}
struct TransactionView<C: 'static + Currency> {
    model: Transaction<C>,
}
impl<C: 'static + Currency> From<Transaction<C>> for TransactionView<C> {
    fn from(transaction: Transaction<C>) -> Self {
        Self {
            model: transaction,
        }
    }
}
impl<C: 'static + Currency> yew::Component for TransactionView<C> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self::from(Transaction::default())
    }
    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }
    fn view(&self) -> yew::Html {
        html!{
            <tr>
                <td>{self.model.get_date().map(|d| format!("{}", d)).unwrap_or("unknown".into())}</td>
                <td>{self.model.get_amount().to_string()}</td>
                <td>{self.model.get_recipient().map(|s| s.to_string()).unwrap_or("None".into())}</td>
                <td>{self.model.get_purposes().map(|ps| ps.to_string()).unwrap_or("None".into())}</td>
                </tr>
        }
    }
}
